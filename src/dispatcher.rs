use enumset::EnumSet;
use smallvec::{smallvec, Array, SmallVec};

use crate::{
    cards::{event::DefaultCardImpl, ids::*},
    cmd_list,
    data_structures::ActionList,
    dispatcher_ops::*,
    phc,
    prelude::ByPlayer,
    types::{
        card_defs::Cost,
        card_impl::{CardImpl, CardImplContext},
        command::*,
        game_state::*,
        input::{Input, NondetResult, PlayerAction},
        logging::Event,
    },
    vector,
};

const SWITCHING_COST: Cost = Cost {
    elem_cost: None,
    unaligned_cost: 1,
    aligned_cost: 0,
    energy_cost: 0,
};

#[inline]
pub(crate) fn cmd_trigger_event_src(
    src_player_id: PlayerId,
    event_id: EventId,
    src: CommandSource,
) -> (CommandContext, Command) {
    (
        CommandContext::new(src_player_id, src, None),
        Command::TriggerEvent(event_id),
    )
}

#[inline]
pub(crate) fn cmd_trigger_event(src_player_id: PlayerId, event_id: EventId) -> (CommandContext, Command) {
    cmd_trigger_event_src(src_player_id, event_id, CommandSource::Event)
}

impl<P: GameStateParams> GameState<P> {
    fn ensure_active_char(&self) -> Result<(PlayerId, u8), DispatchError> {
        let ap = self.phase.active_player();
        if let Some(active_player_id) = ap {
            Ok((active_player_id, self.player(active_player_id).active_char_idx))
        } else {
            Err(DispatchError::InvalidInput(
                "Cannot perform this action outside of Action Phase.",
            ))
        }
    }

    fn end_round(
        &mut self,
        first_end_round: Option<PlayerId>,
        active_player: PlayerId,
    ) -> Result<DispatchResult, DispatchError> {
        self.exec_commands(&cmd_list![cmd_trigger_event(active_player, EventId::DeclareEndOfRound)])?;
        match first_end_round {
            None => {
                let next_player = active_player.opposite();
                self.set_phase(Phase::ActionPhase {
                    first_end_round: Some(active_player),
                    active_player: next_player,
                });
                self.exec_commands(&cmd_list![cmd_trigger_event(next_player, EventId::BeforeAction)])?;
                if let Some(log) = &mut self.log {
                    log.log(Event::Phase(self.phase));
                }
                Ok(DispatchResult::PlayerInput(next_player))
            }
            Some(next_first_active_player) => {
                self.set_phase(Phase::EndPhase {
                    next_first_active_player,
                });
                if let Some(log) = &mut self.log {
                    log.log(Event::Phase(self.phase));
                }
                Ok(DispatchResult::NoInput)
            }
        }
    }

    fn switch_character(&mut self, tgt_char_idx: u8) -> Result<Option<DispatchResult>, DispatchError> {
        let (player_id, char_idx) = self.ensure_active_char()?;

        self.ensure_can_switch_to(player_id, tgt_char_idx)?;

        self.pay_cost(
            &SWITCHING_COST,
            CostType::Switching {
                dst_char_idx: tgt_char_idx,
            },
        )?;
        let ctx = CommandContext::new(
            player_id,
            CommandSource::Switch {
                from_char_idx: char_idx,
                dst_char_idx: tgt_char_idx,
            },
            None,
        );

        if self.try_switch_is_fast_action(player_id, char_idx) {
            self.exec_commands(&vector![(ctx, Command::SwitchCharacter(tgt_char_idx)),])
        } else {
            self.exec_commands(&vector![
                (ctx, Command::SwitchCharacter(tgt_char_idx)),
                (ctx, Command::HandOverPlayer)
            ])
        }
    }

    fn can_switch_to(&self, player_id: PlayerId, char_idx: u8) -> bool {
        self.ensure_can_switch_to(player_id, char_idx).is_ok()
    }

    fn ensure_can_switch_to(&self, player_id: PlayerId, char_idx: u8) -> Result<(), DispatchError> {
        if Some(player_id) != self.phase.active_player() {
            return Err(DispatchError::CannotSwitchInto);
        }

        let player = self.player(player_id);
        if player.active_char_idx == char_idx || !player.is_valid_char_idx(char_idx) {
            return Err(DispatchError::CannotSwitchInto);
        }

        if self.ignore_costs
            || player.can_pay_dice_cost(
                self.status_collections.get(player_id),
                &SWITCHING_COST,
                CostType::Switching { dst_char_idx: char_idx },
            )
        {
            Ok(())
        } else {
            Err(DispatchError::UnableToPayCost)
        }
    }

    fn can_cast_skill(&self, player_id: PlayerId, skill_id: SkillId) -> bool {
        if Some(player_id) != self.phase.active_player() {
            return false;
        }
        let player = self.player(player_id);
        if !player.is_valid_char_idx(player.active_char_idx) {
            return false;
        }
        let status_collection = self.status_collections.get(player_id);
        if status_collection.cannot_perform_actions(player.active_char_idx) {
            return false;
        }

        let active_char = player.active_character();
        let char_skills = &active_char.char_id.char_card().skills;
        if !char_skills.iter().any(|&s| s == skill_id) {
            return false;
        }

        if self.ignore_costs {
            return true;
        }
        let cost = skill_id.skill().cost;
        if !active_char.can_pay_energy_cost(&cost) {
            return false;
        }

        player.can_pay_dice_cost(status_collection, &cost, CostType::Skill(skill_id))
    }

    fn cmd_tgt(&self, player_id: PlayerId) -> Option<CommandTarget> {
        let opp = player_id.opposite();
        let tgt_char_idx = self.player(opp).active_char_idx;
        Some(CommandTarget {
            player_id: opp,
            char_idx: tgt_char_idx,
        })
    }

    /// Requires active player and active character
    fn cast_skill(&mut self, skill_id: SkillId, from_prepare: bool) -> Result<Option<DispatchResult>, DispatchError> {
        let (player_id, char_idx) = self.ensure_active_char()?;
        let char_id = self.player(player_id).active_character().char_id;
        let char_skills = &char_id.char_card().skills;
        if !from_prepare {
            let found = char_skills.iter().any(|&s| s == skill_id);
            if !found {
                return Err(DispatchError::InvalidSkillId);
            }
        }

        if self
            .status_collections
            .get(player_id)
            .cannot_perform_actions(self.player(player_id).active_char_idx)
        {
            return Err(DispatchError::CannotCastSkills);
        }

        let skill = skill_id.skill();
        self.pay_cost(&skill.cost, CostType::Skill(skill_id))?;
        let ctx = {
            let cmd_src = CommandSource::Skill { char_idx, skill_id };
            let cmd_tgt = self.cmd_tgt(player_id);
            CommandContext::new(player_id, cmd_src, cmd_tgt)
        };
        let res = self.exec_commands(&cmd_list![(ctx, Command::CastSkill(skill_id))])?;
        if skill_id == SkillId::FrostflakeArrow {
            let player = self.players.get_mut(player_id);
            player.insert_flag(phc!(self, player_id), PlayerFlag::SkillCastedThisMatch);
        }
        Ok(res)
    }

    fn can_play_card(&self, card_id: CardId, selection: Option<CardSelection>) -> bool {
        let card = card_id.card();
        let Some(player) = self.active_player() else {
            return false;
        };

        if !player.hand.contains(&card_id) {
            return false;
        }

        let active_player_id = self
            .phase
            .active_player()
            .expect("can_play_card: must have active player");
        if !self.ignore_costs {
            let status_collection = self.status_collections.get(active_player_id);
            if !player.can_pay_dice_cost(status_collection, &card.cost, CostType::Card(card_id)) {
                return false;
            }

            if card.cost.energy_cost > 0 {
                let target_char = self.players.get(active_player_id).active_character();
                if !target_char.can_pay_energy_cost(&card.cost) {
                    return false;
                }
            }
        }

        if let Some(ci) = card_id.card_impl() {
            if !self.validate_selection(active_player_id, ci.selection(), selection) {
                return false;
            }

            let cic = CardImplContext {
                players: &self.players,
                status_collections: &self.status_collections,
                active_player_id,
                card_id,
                card,
                selection,
            };
            ci.can_be_played(&cic).to_bool()
        } else {
            selection.is_none()
        }
    }

    fn play_card(
        &mut self,
        card_id: CardId,
        selection: Option<CardSelection>,
    ) -> Result<Option<DispatchResult>, DispatchError> {
        let card = card_id.card();
        let Some(active_player_id) = self.phase.active_player() else {
            return Err(DispatchError::UnableToPlayCard);
        };

        if !self
            .players
            .get_mut(active_player_id)
            .try_remove_card_from_hand(phc!(self, active_player_id), card_id)
        {
            return Err(DispatchError::CardNotOnHand);
        }

        let ctx = CommandContext::new(
            active_player_id,
            CommandSource::Card {
                card_id,
                target: selection,
            },
            self.cmd_tgt(active_player_id),
        );
        self.pay_cost(&card.cost, CostType::Card(card_id))?;
        let cic = CardImplContext {
            players: &self.players,
            status_collections: &self.status_collections,
            active_player_id,
            card_id,
            card,
            selection,
        };
        let mut cmds = cmd_list![];
        if let Some(ci) = card_id.card_impl() {
            ci.can_be_played(&cic).to_result()?;
            if !self.validate_selection(active_player_id, ci.selection(), selection) {
                return Err(DispatchError::InvalidSelection);
            }

            ci.effects(&cic, &ctx, &mut cmds);
        } else {
            if selection.is_some() {
                return Err(DispatchError::InvalidSelection);
            }

            DefaultCardImpl().effects(&cic, &ctx, &mut cmds);
        }

        self.exec_commands(&cmds)
    }

    fn validate_selection(
        &self,
        active_player_id: PlayerId,
        spec: Option<CardSelectionSpec>,
        sel: Option<CardSelection>,
    ) -> bool {
        match (spec, sel) {
            (None, None) => true,
            (None, Some(..)) => false,
            (Some(..), None) => false,
            (Some(spec), Some(sel)) => {
                spec.validate_selection(sel, &self.players, &self.status_collections, active_player_id)
            }
        }
    }

    fn can_perform_elemental_tuning(&self, card_id: CardId) -> bool {
        let Some(active_player_id) = self.phase.active_player() else {
            return false;
        };
        let player = self.player(active_player_id);
        let ep = player.element_priority();
        if player.dice.select_for_elemental_tuning(&ep).is_none() {
            return false;
        }

        player.hand.contains(&card_id)
    }

    fn elemental_tuning(&mut self, player_id: PlayerId, card_id: CardId) -> Result<(), DispatchError> {
        let player = self.players.get_mut(player_id);
        let char_card = player.active_character().char_id.char_card();
        let ep = player.element_priority();
        let Some(elem_to_remove) = player.dice.select_for_elemental_tuning(&ep) else {
            return Err(DispatchError::UnableToPayCost);
        };

        if !player.try_remove_card_from_hand(phc!(self, player_id), card_id) {
            return Err(DispatchError::CardNotOnHand);
        }

        player.update_dice_for_elemental_tuning(phc!(self, player_id), elem_to_remove, char_card.elem);
        Ok(())
    }

    fn available_card_selections(&self, card_id: CardId) -> SmallVec<[Option<CardSelection>; 4]> {
        let Some(player_id) = self.phase.active_player() else {
            return smallvec![None];
        };
        let Some(ci) = card_id.card_impl() else {
            return smallvec![None];
        };

        ci.selection()
            .map(|s| {
                s.available_selections(&self.players, &self.status_collections, player_id)
                    .iter()
                    .copied()
                    .map(Option::Some)
                    .collect()
            })
            .unwrap_or_else(|| smallvec![None])
    }

    /// Determine the cost of the action and whether it is a Fast Action
    /// Precondition (not checked): `self.available_actions().contains(input)`
    pub fn action_info(&self, input: Input) -> (Cost, bool) {
        const NONE: (Cost, bool) = (Cost::ZERO, true);
        let Some(active_player_id) = self.phase.active_player() else {
            return NONE;
        };
        match input {
            Input::NoAction => NONE,
            Input::NondetResult(_) => NONE,
            Input::FromPlayer(p, _) if p != active_player_id => NONE,
            Input::FromPlayer(player_id, action) => {
                if let PlayerAction::ElementalTuning(_) = action {
                    return (Cost::ONE, true);
                };

                let player = self.player(player_id);
                let mut is_fast_action = true;
                let Some((mut cost, cost_type)) = (match action {
                    PlayerAction::PlayCard(card_id, _) => Some((card_id.card().cost, CostType::Card(card_id))),
                    PlayerAction::CastSkill(skill_id) => {
                        is_fast_action = false;
                        Some((skill_id.skill().cost, CostType::Skill(skill_id)))
                    }
                    PlayerAction::SwitchCharacter(dst_char_idx) => {
                        is_fast_action = self.check_switch_is_fast_action(active_player_id, player.active_char_idx);
                        Some((Cost::ONE, CostType::Switching { dst_char_idx }))
                    }
                    PlayerAction::EndRound => return (Cost::ZERO, false),
                    _ => None,
                }) else {
                    return NONE;
                };

                player.augment_cost_immutable(self.status_collections.get(active_player_id), &mut cost, cost_type);

                (cost, is_fast_action)
            }
        }
    }

    /// Get the available game state advancement actions.
    pub fn available_actions(&self) -> ActionList<Input> {
        if let Some(pc) = &self.pending_cmds {
            return pc.suspended_state.available_actions(self);
        }

        let mut acts = smallvec![];

        match self.phase {
            Phase::Drawing { .. } => {
                // nothing
            }
            Phase::SelectStartingCharacter { state } => {
                let player_id = state.active_player();
                for (char_idx, _) in self.player(player_id).char_states.enumerate_valid() {
                    acts.push(Input::FromPlayer(player_id, PlayerAction::SwitchCharacter(char_idx)));
                }
            }
            Phase::ActionPhase {
                active_player: player_id,
                ..
            } => {
                self.available_actions_action_phase(player_id, &mut acts);
            }
            Phase::WinnerDecided { .. } | Phase::EndPhase { .. } | Phase::RollPhase { .. } => {
                acts.push(Input::NoAction);
            }
        }

        acts
    }

    fn available_actions_action_phase(&self, player_id: PlayerId, acts: &mut ActionList<Input>) {
        let player = self.player(player_id);
        let init_acts_len = acts.len();

        if self
            .status_collections
            .get(player_id)
            .is_preparing_skill(player.active_char_idx)
        {
            return;
        }

        // Cast Skill
        if let Some(active_char) = self.active_character() {
            self.available_actions_cast_skill(active_char, player_id, acts);
        }

        // Play Card
        self.available_actions_play_card(player_id, acts);

        let has_others = acts.len() > init_acts_len;

        // Switch
        self.available_actions_switch(player_id, acts);

        // Elemental Tuning
        let allowed_to_et = !player.is_tactical() || !has_others;
        if allowed_to_et && self.active_character().is_some() {
            self.available_actions_et(player_id, acts);
        }

        acts.push(Input::FromPlayer(player_id, PlayerAction::EndRound));
    }

    fn available_actions_play_card<A: Array<Item = Input>>(&self, player_id: PlayerId, acts: &mut SmallVec<A>) {
        let player = self.player(player_id);
        let mut found = EnumSet::default();
        for &card_id in player.hand.iter() {
            if found.contains(card_id) {
                continue;
            }

            for selection in self.available_card_selections(card_id) {
                if self.can_play_card(card_id, selection) {
                    acts.push(Input::FromPlayer(player_id, PlayerAction::PlayCard(card_id, selection)));
                }
                found.insert(card_id);
            }
        }
    }

    fn available_actions_cast_skill<A: Array<Item = Input>>(
        &self,
        active_char: &CharState,
        player_id: PlayerId,
        acts: &mut SmallVec<A>,
    ) {
        let skills = active_char.char_id.char_card().skills;
        for &skill_id in skills.iter().rev() {
            if self.can_cast_skill(player_id, skill_id) {
                acts.push(Input::FromPlayer(player_id, PlayerAction::CastSkill(skill_id)));
            }
        }
    }

    fn available_actions_switch<A: Array<Item = Input>>(&self, player_id: PlayerId, acts: &mut SmallVec<A>) {
        let player = self.player(player_id);
        for (char_idx, _) in player.char_states.enumerate_valid() {
            if self.can_switch_to(player_id, char_idx) {
                acts.push(Input::FromPlayer(player_id, PlayerAction::SwitchCharacter(char_idx)));
            }
        }
    }

    fn available_actions_et<A: Array<Item = Input>>(&self, player_id: PlayerId, acts: &mut SmallVec<A>) {
        let player = self.player(player_id);
        let mut found = EnumSet::default();
        for &card_id in player.hand.iter() {
            if found.contains(card_id) {
                continue;
            }

            if self.can_perform_elemental_tuning(card_id) {
                acts.push(Input::FromPlayer(player_id, PlayerAction::ElementalTuning(card_id)));
                found.insert(card_id);
            }
        }
    }

    fn apply_passives(&mut self) {
        for player_id in PlayerId::VALUES {
            let player = self.player_mut(player_id);
            let mut to_apply = vector![];
            for (i, c) in player.char_states.enumerate_valid() {
                if let Some(p) = &c.char_id.char_card().passive {
                    for &status_id in p.apply_statuses.iter() {
                        to_apply.push((i, status_id));
                    }
                }
            }

            for (char_idx, status_id) in to_apply {
                self.exec_commands(&cmd_list![(
                    CommandContext::new(player_id, CommandSource::Event, None),
                    Command::ApplyCharacterStatus(status_id, char_idx.into()),
                )])
                .expect("apply_passives: myst have no errors");
            }
        }
    }

    pub(crate) fn trigger_switch_cmd(
        player_id: PlayerId,
        from_char_idx: u8,
        dst_char_idx: u8,
    ) -> (CommandContext, Command) {
        let src = CommandSource::Switch {
            from_char_idx,
            dst_char_idx,
        };
        cmd_trigger_event_src(player_id, EventId::Switched, src)
    }

    pub fn to_move_player(&self) -> Option<PlayerId> {
        if let Some(pc) = self.pending_cmds.as_ref() {
            return pc.suspended_state.to_move_player();
        }

        match self.phase {
            Phase::Drawing { .. } => None,
            Phase::SelectStartingCharacter { state } => Some(state.active_player()),
            Phase::RollPhase { .. } => None,
            Phase::ActionPhase { active_player, .. } => {
                let player = self.players.get(active_player);
                if self
                    .status_collections
                    .get(active_player)
                    .is_preparing_skill(player.active_char_idx)
                {
                    None
                } else {
                    Some(active_player)
                }
            }
            Phase::EndPhase { .. } => None,
            Phase::WinnerDecided { .. } => None,
        }
    }

    pub fn nondet_request(&self) -> Option<NondetRequest> {
        if let Some(pending_cmds) = &self.pending_cmds {
            return pending_cmds.suspended_state.nondet_request();
        }

        match self.phase {
            Phase::Drawing { .. } => {
                let n = if self.round_number == 1 { 5 } else { 2 };
                Some(NondetRequest::DrawCards((n, n).into()))
            }
            Phase::RollPhase { roll_phase_state, .. } => match roll_phase_state {
                RollPhaseState::Start => None,
                RollPhaseState::Rolling => Some(NondetRequest::RollDice(
                    (
                        self.player(PlayerId::PlayerFirst)
                            .dice_distribution(self.status_collections.get(PlayerId::PlayerFirst)),
                        self.player(PlayerId::PlayerSecond)
                            .dice_distribution(self.status_collections.get(PlayerId::PlayerSecond)),
                    )
                        .into(),
                )),
            },
            _ => None,
        }
    }

    /// Dispatch a player input and update the game state accordingly.
    /// Postcondition: If the result is Err, then the game state is invalidated.
    pub fn advance(&mut self, input: Input) -> Result<DispatchResult, DispatchError> {
        if input != Input::NoAction {
            if let Some(log) = &mut self.log {
                log.log(Event::Action(input));
            }
        }
        if self.pending_cmds.is_some() {
            let res = self.resolve_pending_cmds(input).map(|opt| self.handle_post_exec(opt));
            // TODO actually implement pending commands incremental hashing
            self.rehash();
            // self.update_hash();
            return res;
        }

        let res = match self.phase {
            Phase::Drawing { first_active_player } => self.advance_drawing_phase(input, first_active_player),
            Phase::SelectStartingCharacter { state } => self.advance_select_starting(input, state),
            Phase::RollPhase {
                first_active_player: active_player,
                roll_phase_state,
            } => self.advance_roll_phase(input, active_player, roll_phase_state),
            Phase::ActionPhase {
                first_end_round,
                active_player,
            } => self.advance_action_phase(input, active_player, first_end_round),
            Phase::EndPhase {
                next_first_active_player: first_active_player,
            } => self.advance_end_phase(input, first_active_player),
            Phase::WinnerDecided { winner } => Ok(DispatchResult::Winner(winner)),
        };
        self.update_hash();
        res
    }

    fn advance_drawing_phase(
        &mut self,
        input: Input,
        first_active_player: PlayerId,
    ) -> Result<DispatchResult, DispatchError> {
        match input {
            Input::NondetResult(NondetResult::ProvideCards(ByPlayer(cards1, cards2))) => {
                self.add_cards_to_hand(PlayerId::PlayerFirst, &cards1);
                self.add_cards_to_hand(PlayerId::PlayerSecond, &cards2);
                if self.round_number == 1 {
                    self.set_phase(Phase::SelectStartingCharacter {
                        state: SelectStartingCharacterState::Start {
                            to_select: first_active_player,
                        },
                    });
                    Ok(DispatchResult::PlayerInput(first_active_player))
                } else {
                    self.set_phase(Phase::RollPhase {
                        first_active_player,
                        roll_phase_state: RollPhaseState::Start,
                    });
                    Ok(DispatchResult::NoInput)
                }
            }
            Input::NondetResult(..) => Err(DispatchError::NondetResultInvalid),
            Input::FromPlayer(..) | Input::NoAction => Err(DispatchError::NondetResultRequired),
        }
    }

    fn advance_select_starting(
        &mut self,
        input: Input,
        state: SelectStartingCharacterState,
    ) -> Result<DispatchResult, DispatchError> {
        let active_player = state.active_player();
        match input {
            Input::NondetResult(..) => Err(DispatchError::NondetResultNotAllowed),
            Input::FromPlayer(player_id, ..) if player_id != active_player => Err(DispatchError::InvalidPlayer),
            Input::FromPlayer(player_id, PlayerAction::SwitchCharacter(char_idx)) => {
                let player = self.players.get(player_id);
                if !player.is_valid_char_idx(char_idx) {
                    return Err(DispatchError::CannotSwitchInto);
                }
                let prev_char_idx = player.active_char_idx;
                if prev_char_idx != char_idx {
                    // use crate::chc;
                    let player = self.players.get_mut(player_id);
                    // player.char_states[prev_char_idx].remove_flag_hashed(chc!(self, player_id, prev_char_idx), CharFlag::PlungingAttack);
                    // player.char_states[char_idx].insert_flag_hashed(chc!(self, player_id, char_idx), CharFlag::PlungingAttack);
                    player.set_active_char_idx(phc!(self, player_id), char_idx);
                }

                match state {
                    SelectStartingCharacterState::Start { to_select } => {
                        self.set_phase(Phase::SelectStartingCharacter {
                            state: SelectStartingCharacterState::FirstSelected {
                                to_select: to_select.opposite(),
                            },
                        });
                        Ok(DispatchResult::PlayerInput(player_id.opposite()))
                    }
                    SelectStartingCharacterState::FirstSelected { to_select } => {
                        self.set_phase(Phase::RollPhase {
                            first_active_player: to_select.opposite(),
                            roll_phase_state: Default::default(),
                        });
                        Ok(DispatchResult::NoInput)
                    }
                }
            }
            _ => Err(DispatchError::InvalidInput("Must select a starting character.")),
        }
    }

    fn advance_roll_phase(
        &mut self,
        input: Input,
        active_player: PlayerId,
        roll_phase_state: RollPhaseState,
    ) -> Result<DispatchResult, DispatchError> {
        match roll_phase_state {
            RollPhaseState::Start => match input {
                Input::NondetResult(..) => Err(DispatchError::NondetResultNotAllowed),
                Input::FromPlayer(..) => Err(DispatchError::InvalidInput("RollPhase: Action is not allowed.")),
                Input::NoAction => {
                    if self.round_number == 1 {
                        self.apply_passives();
                        let Ok(None) = self.exec_commands(&cmd_list![
                            Self::trigger_switch_cmd(
                                PlayerId::PlayerFirst,
                                0,
                                self.players.get(PlayerId::PlayerFirst).active_char_idx
                            ),
                            Self::trigger_switch_cmd(
                                PlayerId::PlayerSecond,
                                0,
                                self.players.get(PlayerId::PlayerSecond).active_char_idx
                            ),
                        ]) else {
                            panic!("advance_roll_phase: Start: Round number is 1 and initial switch triggers failed.")
                        };
                    }
                    if let Some(log) = &mut self.log {
                        log.log(Event::Round(self.round_number, active_player));
                    }
                    self.set_phase(Phase::RollPhase {
                        first_active_player: active_player,
                        roll_phase_state: RollPhaseState::Rolling,
                    });
                    Ok(DispatchResult::NondetRequest(
                        self.nondet_request().expect("nondet_request"),
                    ))
                }
            },
            RollPhaseState::Rolling => match input {
                Input::NondetResult(NondetResult::ProvideDice(ByPlayer(dice1, dice2))) => {
                    self.players.0.update_incremental_element_priority();
                    self.players.1.update_incremental_element_priority();
                    self.players.0.add_dice(phc!(self, PlayerId::PlayerFirst), &dice1);
                    self.players.1.add_dice(phc!(self, PlayerId::PlayerSecond), &dice2);
                    self.exec_commands(&cmd_list![
                        cmd_trigger_event(active_player, EventId::StartOfActionPhase),
                        cmd_trigger_event(active_player.opposite(), EventId::StartOfActionPhase),
                        (CommandContext::new_event(active_player), Command::HandOverPlayer),
                    ])
                    .map(|opt| self.handle_post_exec(opt))
                    .expect("advance_roll_phase: Rolling: failed to execute initialize commands");

                    if self.players.0.is_tactical() {
                        self.perform_pseudo_elemental_tuning(PlayerId::PlayerFirst);
                    }

                    if self.players.1.is_tactical() {
                        self.perform_pseudo_elemental_tuning(PlayerId::PlayerSecond);
                    }
                    Ok(DispatchResult::PlayerInput(active_player))
                }
                Input::NondetResult(..) => Err(DispatchError::NondetResultInvalid),
                Input::FromPlayer(..) | Input::NoAction => Err(DispatchError::NondetResultRequired),
            },
        }
    }

    fn handle_preparing_skill(
        &mut self,
        input: Input,
        active_player_id: PlayerId,
    ) -> Option<Result<DispatchResult, DispatchError>> {
        let player = self.players.get(active_player_id);
        let status_collection = self.status_collections.get(active_player_id);
        let Some((skill_id, key, turns_remaining)) =
            status_collection.find_preparing_skill_with_status_key_and_turns_remaining()
        else {
            return None;
        };
        let active_char_idx = player.active_char_idx;
        let Input::NoAction = input else {
            return Some(Err(DispatchError::InvalidInput("Preparing skill")));
        };
        let char_idx = key.char_idx().expect("Prepared skills must be character statuses.");
        if active_char_idx != char_idx {
            self.status_collections
                .mutate_hashed(phc!(self, active_player_id), |sc| {
                    sc.delete(key);
                });
            // Character switched away -> cancel preparation and delete status
            let res = self
                .exec_commands(&cmd_list![(
                    CommandContext::new_event(active_player_id),
                    Command::HandOverPlayer
                )])
                .map(|opt| self.handle_post_exec(opt));
            return Some(res);
        }

        let res = if turns_remaining == 0 {
            self.status_collections
                .mutate_hashed(phc!(self, active_player_id), |sc| {
                    sc.delete(key);
                });
            if self
                .status_collections
                .get(active_player_id)
                .cannot_perform_actions(char_idx)
            {
                return Some(Ok(self.handle_post_exec(None)));
            }
            self.cast_skill(skill_id, true).map(|opt| self.handle_post_exec(opt))
        } else {
            self.status_collections
                .mutate_hashed(phc!(self, active_player_id), |sc| {
                    let state = sc.get_mut(key).expect("Status key must exist.");
                    state.set_counter(turns_remaining - 1);
                });
            self.exec_commands(&cmd_list![(
                CommandContext::new_event(active_player_id),
                Command::HandOverPlayer
            )])
            .map(|opt| self.handle_post_exec(opt))
        };
        Some(res)
    }

    fn advance_action_phase(
        &mut self,
        input: Input,
        active_player: PlayerId,
        first_end_round: Option<PlayerId>,
    ) -> Result<DispatchResult, DispatchError> {
        if let Some(res) = self.handle_preparing_skill(input, active_player) {
            return res;
        }
        match input {
            Input::NondetResult(..) => Err(DispatchError::NondetResultNotAllowed),
            Input::NoAction => Err(DispatchError::InvalidPlayer),
            Input::FromPlayer(p1, _) if p1 != active_player => Err(DispatchError::InvalidPlayer),
            Input::FromPlayer(_, action) => match action {
                PlayerAction::EndRound => self.end_round(first_end_round, active_player),
                PlayerAction::CastSkill(skill_id) => {
                    self.cast_skill(skill_id, false).map(|opt| self.handle_post_exec(opt))
                }
                PlayerAction::SwitchCharacter(idx) => self.switch_character(idx).map(|opt| self.handle_post_exec(opt)),
                PlayerAction::ElementalTuning(card_id) => self
                    .elemental_tuning(active_player, card_id)
                    .map(|_| DispatchResult::PlayerInput(active_player)),
                PlayerAction::PlayCard(card_id, target) => {
                    self.play_card(card_id, target).map(|opt| self.handle_post_exec(opt))
                }
                PlayerAction::PostDeathSwitch(_) => Err(DispatchError::CannotSwitchInto),
            },
        }
    }

    fn advance_end_phase(
        &mut self,
        input: Input,
        first_active_player: PlayerId,
    ) -> Result<DispatchResult, DispatchError> {
        match input {
            Input::NondetResult(..) => Err(DispatchError::NondetResultNotAllowed),
            Input::NoAction => {
                let p1 = first_active_player;
                let p2 = first_active_player.opposite();
                self.exec_commands(&cmd_list![
                    cmd_trigger_event(p1, EventId::EndPhase),
                    cmd_trigger_event(p2, EventId::EndPhase),
                    cmd_trigger_event(p1, EventId::EndOfTurn),
                    cmd_trigger_event(p2, EventId::EndOfTurn),
                    (CommandContext::new(p1, CommandSource::Event, None), Command::EndOfTurn),
                ])
                .map(|opt| self.handle_post_exec(opt))
            }
            _ => Err(DispatchError::InvalidInput("EndPhase: Invalid input")),
        }
    }
}
