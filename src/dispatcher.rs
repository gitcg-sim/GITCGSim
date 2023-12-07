use smallvec::{smallvec, SmallVec};

use crate::cards::event::DefaultCardImpl;
use crate::cards::ids::lookup::GetCard;
use crate::cards::ids::{CardId, GetCharCard, GetSkill};
use crate::data_structures::ActionList;
use crate::dispatcher_ops::exec_command_helpers::{augment_cost_immutable, can_pay_dice_cost};
use crate::dispatcher_ops::types::NondetRequest;
use crate::types::card_impl::{CardImpl, CardImplContext};
use crate::types::input::{Input, NondetResult, PlayerAction};
use crate::{cards::ids::SkillId, cmd_list, vector};
use crate::{mutate_statuses, phc};

use crate::dispatcher_ops::types::*;
use crate::types::command::*;
use crate::types::game_state::*;
use crate::types::{card_defs::Cost, logging::Event};

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

impl GameState {
    fn ensure_active_char(&self) -> Result<(PlayerId, u8), DispatchError> {
        let ap = self.phase.active_player();
        if let Some(active_player_id) = ap {
            Ok((active_player_id, self.get_player(active_player_id).active_char_idx))
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
                self.log.log(Event::Phase(self.phase));
                Ok(DispatchResult::PlayerInput(next_player))
            }
            Some(next_first_active_player) => {
                self.set_phase(Phase::EndPhase {
                    next_first_active_player,
                });
                self.log.log(Event::Phase(self.phase));
                Ok(DispatchResult::NoInput)
            }
        }
    }

    fn switch_character(&mut self, tgt_char_idx: u8) -> Result<Option<DispatchResult>, DispatchError> {
        let (player_id, char_idx) = self.ensure_active_char()?;

        self.ensure_can_switch_to(player_id, tgt_char_idx)?;

        self.pay_cost(&SWITCHING_COST, CostType::Switching)?;
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

        let player = self.get_player(player_id);
        if player.active_char_idx == char_idx || !player.is_valid_char_idx(char_idx) {
            return Err(DispatchError::CannotSwitchInto);
        }

        if self.ignore_costs || can_pay_dice_cost(player, &SWITCHING_COST, CostType::Switching) {
            Ok(())
        } else {
            Err(DispatchError::UnableToPayCost)
        }
    }

    fn can_cast_skill(&self, player_id: PlayerId, skill_id: SkillId) -> bool {
        if Some(player_id) != self.phase.active_player() {
            return false;
        }
        let player = self.get_player(player_id);
        if !player.is_valid_char_idx(player.active_char_idx) {
            return false;
        }
        if player.status_collection.cannot_perform_actions(player.active_char_idx) {
            return false;
        }

        let active_char = player.get_active_character();
        let char_skills = &active_char.char_id.get_char_card().skills;
        if !char_skills.to_vec().iter().any(|&s| s == skill_id) {
            return false;
        }

        if self.ignore_costs {
            return true;
        }
        let cost = skill_id.get_skill().cost;
        if !active_char.can_pay_energy_cost(&cost) {
            return false;
        }

        can_pay_dice_cost(player, &cost, CostType::Skill(skill_id))
    }

    fn cmd_tgt(&self, player_id: PlayerId) -> Option<CommandTarget> {
        let opp = player_id.opposite();
        let tgt_char_idx = self.get_player(opp).active_char_idx;
        Some(CommandTarget {
            player_id: opp,
            char_idx: tgt_char_idx,
        })
    }

    /// Requires active player and active character
    fn cast_skill(&mut self, skill_id: SkillId, from_prepare: bool) -> Result<Option<DispatchResult>, DispatchError> {
        let (player_id, char_idx) = self.ensure_active_char()?;
        let char_id = self.get_player(player_id).get_active_character().char_id;
        let char_skills = &char_id.get_char_card().skills;
        if !from_prepare {
            let found = char_skills.to_vec().iter().any(|&s| s == skill_id);
            if !found {
                return Err(DispatchError::InvalidSkillId);
            }
        }

        if self
            .get_player(player_id)
            .status_collection
            .cannot_perform_actions(self.get_player(player_id).active_char_idx)
        {
            return Err(DispatchError::CannotCastSkills);
        }

        let skill = skill_id.get_skill();
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
        let card = card_id.get_card();
        let Some(player) = self.get_active_player() else {
            return false;
        };

        if !player.hand.contains(&card_id) {
            return false;
        }

        let active_player_id = self.phase.active_player().unwrap();
        if !self.ignore_costs {
            if !can_pay_dice_cost(player, &card.cost, CostType::Card(card_id)) {
                return false;
            }

            if card.cost.energy_cost > 0 {
                let target_char = self.players.get(active_player_id).get_active_character();
                if !target_char.can_pay_energy_cost(&card.cost) {
                    return false;
                }
            }
        }

        if let Some(ci) = card_id.get_card_impl() {
            if !self.validate_selection(active_player_id, ci.selection(), selection) {
                return false;
            }

            let cic = CardImplContext {
                game_state: self,
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
        let card = card_id.get_card();
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
            game_state: self,
            active_player_id,
            card_id,
            card,
            selection,
        };
        let mut cmds = cmd_list![];
        if let Some(ci) = card_id.get_card_impl() {
            ci.can_be_played(&cic).to_result()?;
            if !self.validate_selection(active_player_id, ci.selection(), selection) {
                return Err(DispatchError::InvalidSelection);
            }

            ci.get_effects(&cic, &ctx, &mut cmds);
        } else {
            if selection.is_some() {
                return Err(DispatchError::InvalidSelection);
            }

            DefaultCardImpl().get_effects(&cic, &ctx, &mut cmds);
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
            (Some(spec), Some(sel)) => spec.validate_selection(sel, &self.players, active_player_id),
        }
    }

    fn can_perform_elemental_tuning(&self, card_id: CardId) -> bool {
        let Some(active_player_id) = self.phase.active_player() else {
            return false;
        };
        let player = self.get_player(active_player_id);
        let ep = player.get_element_priority();
        if player.dice.select_for_elemental_tuning(&ep).is_none() {
            return false;
        }

        player.hand.contains(&card_id)
    }

    fn elemental_tuning(&mut self, player_id: PlayerId, card_id: CardId) -> Result<(), DispatchError> {
        let player = self.players.get_mut(player_id);
        let char_card = player.get_active_character().char_id.get_char_card();
        let ep = player.get_element_priority();
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
        let Some(ci) = card_id.get_card_impl() else {
            return smallvec![None];
        };

        ci.selection()
            .map(|s| {
                s.available_selections(&self.players, player_id)
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

                let player = self.get_player(player_id);
                let mut is_fast_action = true;
                let Some((mut cost, cost_type)) = (match action {
                    PlayerAction::PlayCard(card_id, _) => Some((card_id.get_card().cost, CostType::Card(card_id))),
                    PlayerAction::CastSkill(skill_id) => {
                        is_fast_action = false;
                        Some((skill_id.get_skill().cost, CostType::Skill(skill_id)))
                    }
                    PlayerAction::SwitchCharacter(_) => {
                        is_fast_action = self.check_switch_is_fast_action(active_player_id, player.active_char_idx);
                        Some((Cost::ONE, CostType::Switching))
                    }
                    PlayerAction::EndRound => return (Cost::ZERO, false),
                    _ => None,
                }) else {
                    return NONE;
                };

                augment_cost_immutable(player, &mut cost, cost_type);

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
            Phase::SelectStartingCharacter { already_selected } => {
                let player_id = Phase::select_starting_to_move(already_selected);
                for (char_idx, _) in self.get_player(player_id).char_states.enumerate_valid() {
                    if self.can_switch_to(player_id, char_idx) {
                        acts.push(Input::FromPlayer(player_id, PlayerAction::SwitchCharacter(char_idx)));
                    }
                }
            }
            Phase::ActionPhase {
                active_player: player_id,
                ..
            } => {
                self.available_actions_action_phase(player_id, &mut acts);
            }
            Phase::WinnerDecided { .. } => {}
            Phase::EndPhase { .. } => {}
            Phase::RollPhase { .. } => {}
        }

        if acts.is_empty() {
            acts.push(Input::NoAction);
        }

        acts
    }

    fn available_actions_action_phase(&self, player_id: PlayerId, acts: &mut ActionList<Input>) {
        let player = self.get_player(player_id);
        let mut has_others = false;

        // Cast Skill
        if let Some(cs) = self.get_active_character() {
            if player.is_preparing_skill() {
                return;
            }

            let skills = cs.char_id.get_char_card().skills;
            let mut skills_vec = skills.to_vec();
            skills_vec.reverse();
            for skill_id in skills_vec {
                if self.can_cast_skill(player_id, skill_id) {
                    acts.push(Input::FromPlayer(player_id, PlayerAction::CastSkill(skill_id)));
                    has_others = true;
                }
            }
        }

        // Play Card
        {
            let mut found: SmallVec<[CardId; 8]> = SmallVec::new();
            for &card_id in &player.hand {
                if found.contains(&card_id) {
                    continue;
                }

                for selection in self.available_card_selections(card_id) {
                    if self.can_play_card(card_id, selection) {
                        acts.push(Input::FromPlayer(player_id, PlayerAction::PlayCard(card_id, selection)));
                    }
                    found.push(card_id);
                }
            }
            if !found.is_empty() {
                has_others = true;
            }
        }

        // Switch
        for (char_idx, _) in player.char_states.enumerate_valid() {
            if self.can_switch_to(player_id, char_idx) {
                acts.push(Input::FromPlayer(player_id, PlayerAction::SwitchCharacter(char_idx)));
            }
        }

        // Elemental Tuning
        let allowed_to_et = !player.is_tactical() || !has_others;
        if allowed_to_et && self.get_active_character().is_some() {
            let mut found: SmallVec<[CardId; 8]> = SmallVec::new();
            for &card_id in &player.hand {
                if found.contains(&card_id) {
                    continue;
                }

                if self.can_perform_elemental_tuning(card_id) {
                    acts.push(Input::FromPlayer(player_id, PlayerAction::ElementalTuning(card_id)));
                    found.push(card_id);
                }
            }
        }

        acts.push(Input::FromPlayer(player_id, PlayerAction::EndRound));
    }

    fn apply_passives(&mut self) {
        for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
            let player = self.get_player_mut(player_id);
            let mut to_apply = vector![];
            for (i, c) in player.char_states.enumerate_valid() {
                if let Some(p) = &c.char_id.get_char_card().passive {
                    for status_id in p.apply_statuses.to_vec() {
                        to_apply.push((i, status_id));
                    }
                }
            }

            for (char_idx, status_id) in to_apply {
                self.exec_commands(&cmd_list![(
                    CommandContext::new(player_id, CommandSource::Event, None),
                    Command::ApplyCharacterStatus(status_id, char_idx.into()),
                )])
                .unwrap();
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
        if self.pending_cmds.is_some() {
            let pc = self.pending_cmds.clone().unwrap();
            return pc.suspended_state.to_move_player();
        }

        match self.phase {
            Phase::SelectStartingCharacter { already_selected } => {
                Some(Phase::select_starting_to_move(already_selected))
            }
            Phase::RollPhase { .. } => None,
            Phase::ActionPhase { active_player, .. } => {
                let player = self.players.get(active_player);
                if player.is_preparing_skill() {
                    None
                } else {
                    Some(active_player)
                }
            }
            Phase::EndPhase { .. } => None,
            Phase::WinnerDecided { .. } => None,
        }
    }

    pub fn get_nondet_request(&self) -> Option<NondetRequest> {
        if let Some(pending_cmds) = &self.pending_cmds {
            return pending_cmds.suspended_state.get_nondet_request();
        }

        match self.phase {
            Phase::RollPhase { roll_phase_state, .. } => match roll_phase_state {
                RollPhaseState::Start => None,
                RollPhaseState::Drawing => {
                    let n = if self.round_number == 1 { 5 } else { 2 };
                    Some(NondetRequest::DrawCards(n, n))
                }
                RollPhaseState::Rolling => Some(NondetRequest::RollDice(
                    self.get_player(PlayerId::PlayerFirst).get_dice_distribution(),
                    self.get_player(PlayerId::PlayerSecond).get_dice_distribution(),
                )),
            },
            _ => None,
        }
    }

    /// Dispatch a player input and update the game state accordingly.
    /// Postcondition: If the result is Err, then the game state is invalidated.
    pub fn advance(&mut self, input: Input) -> Result<DispatchResult, DispatchError> {
        if self.log.enabled && input != Input::NoAction {
            self.log.log(Event::Action(input));
        }
        if self.pending_cmds.is_some() {
            let res = self.resolve_pending_cmds(input).map(|opt| self.handle_post_exec(opt));
            // TODO actually implement pending commands incremental hashing
            self.rehash();
            // self.update_hash();
            return res;
        }

        let res = match self.phase {
            Phase::SelectStartingCharacter { already_selected } => {
                self.advance_select_starting(input, already_selected)
            }
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

    fn advance_select_starting(
        &mut self,
        input: Input,
        already_selected: Option<PlayerId>,
    ) -> Result<DispatchResult, DispatchError> {
        let active_player = Phase::select_starting_to_move(already_selected);
        match input {
            Input::NondetResult(..) => Err(DispatchError::NondetResultNotAllowed),
            Input::FromPlayer(player_id, ..) if player_id != active_player => Err(DispatchError::InvalidPlayer),
            Input::FromPlayer(player_id, PlayerAction::SwitchCharacter(char_idx)) => {
                let player = &self.players.get(player_id);
                if !player.is_valid_char_idx(char_idx) {
                    return Err(DispatchError::CannotSwitchInto);
                }
                self.get_player_mut(player_id).active_char_idx = char_idx;
                self.phase = match already_selected {
                    None => Phase::SelectStartingCharacter {
                        already_selected: Some(player_id),
                    },
                    Some(..) => Phase::RollPhase {
                        first_active_player: player_id.opposite(),
                        roll_phase_state: Default::default(),
                    },
                };
                Ok(DispatchResult::PlayerInput(player_id.opposite()))
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
                        let None = self
                            .exec_commands(&cmd_list![
                                Self::trigger_switch_cmd(PlayerId::PlayerFirst, 0, 0),
                                Self::trigger_switch_cmd(PlayerId::PlayerSecond, 0, 0),
                            ])
                            .unwrap()
                        else {
                            unreachable!()
                        };
                    }
                    let log = &mut self.log;
                    log.log(Event::Round(self.round_number, active_player));
                    self.set_phase(Phase::RollPhase {
                        first_active_player: active_player,
                        roll_phase_state: RollPhaseState::Drawing,
                    });
                    Ok(DispatchResult::NondetRequest(self.get_nondet_request().unwrap()))
                }
            },
            RollPhaseState::Drawing => {
                match input {
                    Input::NondetResult(NondetResult::ProvideCards(cards1, cards2)) => {
                        self.add_cards_to_hand(PlayerId::PlayerFirst, &cards1);
                        self.add_cards_to_hand(PlayerId::PlayerSecond, &cards2);
                        self.set_phase(Phase::RollPhase {
                            first_active_player: active_player,
                            roll_phase_state: RollPhaseState::Rolling,
                        });
                        // TODO effects that change reroll counts
                        Ok(DispatchResult::NondetRequest(self.get_nondet_request().unwrap()))
                    }
                    Input::NondetResult(..) => Err(DispatchError::NondetResultInvalid),
                    Input::FromPlayer(..) | Input::NoAction => Err(DispatchError::NondetResultRequired),
                }
            }
            RollPhaseState::Rolling => match input {
                Input::NondetResult(NondetResult::ProvideDice(dice1, dice2)) => {
                    self.players.0.add_dice(phc!(self, PlayerId::PlayerFirst), &dice1);
                    self.players.1.add_dice(phc!(self, PlayerId::PlayerSecond), &dice2);
                    self.exec_commands(&cmd_list![
                        cmd_trigger_event(active_player, EventId::StartOfActionPhase),
                        cmd_trigger_event(active_player.opposite(), EventId::StartOfActionPhase),
                        (CommandContext::new_event(active_player), Command::HandOverPlayer),
                    ])
                    .map(|opt| self.handle_post_exec(opt))
                    .unwrap();

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
        let active_char_idx = player.active_char_idx;
        let Some((skill_id, key, turns_remaining)) = player
            .status_collection
            .find_preparing_skill_with_status_key_and_turns_remaining()
        else {
            return None;
        };
        let Input::NoAction = input else {
            return Some(Err(DispatchError::InvalidInput("Preparing skill")));
        };
        let char_idx = key.char_idx().expect("Prepared skills must be character statuses.");
        if active_char_idx != char_idx {
            mutate_statuses!(self, active_player_id, |sc| {
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
            mutate_statuses!(self, active_player_id, |sc| {
                sc.delete(key);
            });
            if self
                .players
                .get(active_player_id)
                .status_collection
                .cannot_perform_actions(char_idx)
            {
                return Some(Ok(self.handle_post_exec(None)));
            }
            self.cast_skill(skill_id, true).map(|opt| self.handle_post_exec(opt))
        } else {
            mutate_statuses!(self, active_player_id, |sc| {
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
