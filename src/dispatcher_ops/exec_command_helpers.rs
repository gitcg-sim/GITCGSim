use super::*;

use crate::cmd_list;
use crate::data_structures::CommandList;
use crate::zobrist_hash::PlayerHashContext;

#[macro_export]
#[doc(hidden)]
macro_rules! view {
    ($p: ident) => {
        PlayerStateView {
            active_char_idx: $p.active_char_idx,
            char_states: &$p.char_states,
            flags: $p.flags,
            dice: $p.dice,
            // TODO will eventually be needed?
            // affected_by: $p.status_collection.get_affected_by_keys(),
            affected_by: Default::default(),
        }
    };
}

/// Mutate a particular `StatusCollection` within the `GameState` in the code block
/// while updating the Zobrish hash before and after the block.
///
/// Do not early terminate inside the block to preserve hash coherency.
#[macro_export]
#[doc(hidden)]
macro_rules! mutate_statuses {
    ($self: expr, $player_id: expr, | $sc: ident | $closure: block) => {{
        let player_id: PlayerId = $player_id;
        let player: &mut _ = $self.players.get_mut(player_id);
        $crate::mutate_statuses_1!(phc!($self, player_id), player, |$sc| $closure)
    }};
}

/// Mutate a particular `StatusCollection` within the `GameState` in the code block
/// while updating the Zobrish hash before and after the block.
///
/// Do not early terminate inside the block to preserve hash coherency.
#[macro_export]
#[doc(hidden)]
macro_rules! mutate_statuses_1 {
    ($c: expr, $player: ident, | $sc: ident | $closure: block) => {{
        let (h, player_id): $crate::zobrist_hash::PlayerHashContext = $c;
        let $sc: &mut $crate::types::game_state::StatusCollection = &mut $player.status_collection;
        $sc.zobrist_hash(h, player_id);
        let _r = $closure;
        $sc.zobrist_hash(h, player_id);
        _r
    }};
}

impl StatusCollection {
    pub fn augment_outgoing_dmg_for_statuses(
        &mut self,
        sicb: StatusImplContextBuilder<DMGInfo>,
        dmg: &mut DealDMG,
    ) -> bool {
        self.consume_statuses(
            sicb.src_char_idx_selector(),
            |si| si.responds_to().contains(RespondsTo::OutgoingDMG),
            |es, sk, si| si.outgoing_dmg(&sicb.build(sk, es), dmg),
        )
    }

    pub fn augment_outgoing_dmg_target_for_statuses(
        &mut self,
        sicb: StatusImplContextBuilder<DMGInfo>,
        tgt_chars: &CharStates,
        tgt_active_char_idx: u8,
        dmg: &DealDMG,
        target_char_idx: &mut u8,
    ) -> bool {
        self.consume_statuses(
            sicb.src_char_idx_selector(),
            |si| si.responds_to().contains(RespondsTo::OutgoingDMG),
            |es, sk, si| {
                si.outgoing_dmg_target(
                    &sicb.build(sk, es),
                    tgt_chars,
                    tgt_active_char_idx,
                    dmg,
                    target_char_idx,
                )
            },
        )
    }

    pub fn augment_late_outgoing_dmg_for_statuses(
        &mut self,
        sicb: StatusImplContextBuilder<DMGInfo>,
        dmg: &mut DealDMG,
    ) -> bool {
        self.consume_statuses(
            sicb.src_char_idx_selector(),
            |si| si.responds_to().contains(RespondsTo::OutgoingDMG),
            |es, sk, si| si.late_outgoing_dmg(&sicb.build(sk, es), dmg),
        )
    }

    pub fn augment_outgoing_reaction_dmg_for_statuses(
        &mut self,
        sicb: StatusImplContextBuilder<DMGInfo>,
        reaction: (Reaction, Option<Element>),
        dmg: &mut DealDMG,
    ) -> bool {
        self.consume_statuses(
            sicb.src_char_idx_selector(),
            |si| si.responds_to().contains(RespondsTo::OutgoingReactionDMG),
            |es, sk, si| si.outgoing_reaction_dmg(&sicb.build(sk, es), reaction, dmg),
        )
    }

    pub fn multiply_outgoing_dmg_for_statuses(
        &mut self,
        sicb: StatusImplContextBuilder<DMGInfo>,
        mult: &mut u8,
    ) -> bool {
        self.consume_statuses(
            sicb.src_char_idx_selector(),
            |si| si.responds_to().contains(RespondsTo::MultiplyOutgoingDMG),
            |es, sk, si| si.multiply_dmg(&sicb.build(sk, es), mult),
        )
    }

    pub fn augment_incoming_dmg_for_statuses(
        &mut self,
        sicb: StatusImplContextBuilder,
        char_idx: u8,
        dmg: &mut DealDMG,
    ) -> bool {
        self.consume_statuses(
            CharIdxSelector::One(char_idx),
            |si| si.responds_to().contains(RespondsTo::IncomingDMG),
            |es, sk, si| si.incoming_dmg(&sicb.build(sk, es), dmg),
        )
    }

    pub fn consume_shield_points_for_statuses(&mut self, char_idx: u8, dmg: &mut DealDMG) -> bool {
        let mut found = false;
        self.for_each_char_status_mut_retain(
            Some(char_idx),
            |status_id, eff_state| {
                let status = status_id.get_status();
                if !(dmg.dmg > 0 && status.usages_as_shield_points) {
                    return true;
                }
                found = true;
                let u = eff_state.get_usages();
                if u > dmg.dmg {
                    let d = dmg.dmg;
                    dmg.dmg = 0;
                    eff_state.set_usages(u - d);
                    true
                } else {
                    // u <= dmg.dmg
                    eff_state.set_usages(0);
                    dmg.dmg -= u;
                    status.manual_discard
                }
            },
            |_, _| {
                // Summons can't have Shield Points
                true
            },
            |_, _| {
                // Supports can't have Shield Points
                true
            },
        );
        found
    }
}

impl CostType {
    #[inline]
    fn into_cmd_src(self, active_char_idx: u8) -> CommandSource {
        match self {
            CostType::Switching {
                dst_char_idx: tgt_char_idx,
            } => CommandSource::Switch {
                from_char_idx: active_char_idx,
                dst_char_idx: tgt_char_idx,
            },
            CostType::Card(card_id) => CommandSource::Card { card_id, target: None },
            CostType::Skill(skill_id) => CommandSource::Skill {
                char_idx: active_char_idx,
                skill_id,
            },
        }
    }
}

impl PlayerState {
    #[inline]
    pub fn is_melee_stance(&self) -> bool {
        self.status_collection
            .has_character_status(self.active_char_idx, StatusId::MeleeStance)
    }

    #[inline]
    pub fn relative_switch_char_idx(&self, switch_type: RelativeCharIdx) -> Option<u8> {
        self.char_states
            .relative_switch_char_idx(self.active_char_idx, switch_type)
    }

    // TODO can reduce cost for character talent cards
    // TODO separate status_collection
    pub fn augment_cost(&mut self, c: PlayerHashContext, cost: &mut Cost, cost_type: CostType) -> bool {
        let char_idx = self.active_char_idx;
        if !self.status_collection.responds_to(RespondsTo::UpdateCost) {
            return false;
        }

        let view = &view!(self);
        mutate_statuses_1!(c, self, |sc| {
            let ctx = &CommandContext::EMPTY.with_src(cost_type.into_cmd_src(self.active_char_idx));
            let sicb = StatusImplContextBuilder::new(view, ctx, ());
            sc.consume_statuses(
                CharIdxSelector::One(char_idx),
                |si| si.responds_to().contains(RespondsTo::UpdateCost),
                |es, sk, si| si.update_cost(&sicb.build(sk, es), cost, cost_type),
            )
        })
    }

    // TODO separate status_collection
    pub fn augment_cost_immutable(&self, cost: &mut Cost, cost_type: CostType) {
        let sc = &self.status_collection;
        if !sc.responds_to(RespondsTo::UpdateCost) {
            return;
        }

        let char_idx = self.active_char_idx;
        let view = &view!(self);
        let ctx = &CommandContext::EMPTY.with_src(cost_type.into_cmd_src(self.active_char_idx));
        let sicb = StatusImplContextBuilder::new(view, ctx, ());
        sc.consume_statuses_immutable(
            CharIdxSelector::One(char_idx),
            |si| si.responds_to().contains(RespondsTo::UpdateCost),
            |es, sk, si| si.update_cost(&sicb.build(sk, es), cost, cost_type),
        );
    }

    // TODO separate status_collection
    pub fn update_gains_energy(&self, ctx_for_skill: &CommandContext, gains_energy: &mut bool) {
        let sc = &self.status_collection;
        if !sc.responds_to(RespondsTo::GainsEnergy) {
            return;
        }

        let char_idx = self.active_char_idx;
        let view = &view!(self);
        let ctx = &CommandContext::EMPTY;
        let sicb = StatusImplContextBuilder::new(view, ctx, ());
        sc.consume_statuses_immutable(
            CharIdxSelector::One(char_idx),
            |si| si.responds_to().contains(RespondsTo::GainsEnergy),
            |es, sk, si| {
                si.gains_energy(&sicb.build(sk, es), ctx_for_skill, gains_energy)
                    .then_some(AppliedEffectResult::NoChange)
            },
        );
    }

    // TODO separate status_collection
    pub fn update_dice_distribution(&self, dist: &mut DiceDistribution) {
        let sc = &self.status_collection;
        if !sc.responds_to(RespondsTo::DiceDistribution) {
            return;
        }

        let view = &view!(self);
        let ctx = &CommandContext::EMPTY;
        let sicb = StatusImplContextBuilder::new(view, ctx, ());
        sc.consume_statuses_immutable(
            // Does not need to be active character to take effect
            CharIdxSelector::All,
            |si| si.responds_to().contains(RespondsTo::DiceDistribution),
            |es, sk, si| {
                si.dice_distribution(&sicb.build(sk, es), dist)
                    .then_some(AppliedEffectResult::NoChange)
            },
        );
    }

    pub fn can_pay_dice_cost(&self, cost: &Cost, cost_type: CostType) -> bool {
        let ep = self.get_element_priority_for_cost_type(cost_type);
        let mut cost = *cost;
        self.augment_cost_immutable(&mut cost, cost_type);
        self.dice.try_pay_cost(&cost, &ep).is_some()
    }

    /// Assumption: augment_cost will never increase costs
    pub fn try_pay_dice_cost(&mut self, c: PlayerHashContext, cost: &Cost, cost_type: CostType) -> Option<DiceCounter> {
        let ep = self.get_element_priority_for_cost_type(cost_type);
        if let Some(d) = self.dice.try_pay_cost(cost, &ep) {
            Some(d)
        } else {
            let mut cost = *cost;
            self.augment_cost(c, &mut cost, cost_type);
            self.dice.try_pay_cost(&cost, &ep)
        }
    }

    pub fn get_cast_skill_cmds(
        &self,
        ctx: &CommandContext,
        skill_id: SkillId,
    ) -> CommandList<(CommandContext, Command)> {
        let src_player = self;
        let skill = skill_id.get_skill();
        let mut cmds: CommandList<(CommandContext, Command)> = cmd_list![];
        if let Some(deal_dmg) = skill.deal_dmg {
            cmds.push((*ctx, Command::DealDMG(deal_dmg)));
        }

        if let Some(status_id) = skill.apply {
            match status_id.get_status().attach_mode {
                StatusAttachMode::Character => {
                    let char_idx = src_player.active_char_idx;
                    cmds.push((*ctx, Command::ApplyCharacterStatus(status_id, char_idx.into())));
                }
                StatusAttachMode::Team => {
                    cmds.push((*ctx, Command::ApplyStatusToTeam(status_id)));
                }
                StatusAttachMode::Summon => panic!("Cannot attach summon status {status_id:?}."),
                StatusAttachMode::Support => panic!("Cannot attach support status {status_id:?}."),
            }
        }

        if let Some(summon_spec) = skill.summon {
            match summon_spec {
                SummonSpec::One(summon_id) => {
                    cmds.push((*ctx, Command::Summon(summon_id)));
                }
                SummonSpec::MultiRandom { count: 0, .. } => {}
                SummonSpec::MultiRandom {
                    summon_ids,
                    count,
                    prioritize_new,
                } => {
                    let existing_summon_ids = if prioritize_new {
                        src_player
                            .status_collection
                            .iter_entries()
                            .filter_map(|k| match k.key {
                                StatusKey::Summon(summon_id) => Some(summon_id),
                                _ => None,
                            })
                            .fold(Default::default(), |s, k| s | k)
                    } else {
                        Default::default()
                    };
                    cmds.push((
                        *ctx,
                        Command::SummonRandom(SummonRandomSpec::new(summon_ids, existing_summon_ids, count)),
                    ));
                }
            }
        }

        for c in skill.commands.to_vec_copy() {
            cmds.push((*ctx, c));
        }

        let mut gains_energy = !skill.no_energy;
        src_player.update_gains_energy(ctx, &mut gains_energy);
        if let Some(si) = skill.skill_impl {
            si.get_commands(src_player, &src_player.status_collection, ctx, &mut cmds);
        }

        if gains_energy && skill.skill_type != SkillType::ElementalBurst {
            cmds.push((*ctx, Command::AddEnergy(1, CmdCharIdx::Active)));
        }

        cmds.push((
            *ctx,
            Command::TriggerXEvent(XEvent::Skill(XEventSkill {
                src_player_id: ctx.src_player_id,
                src_char_idx: src_player.active_char_idx,
                skill_id,
            })),
        ));
        cmds.push((*ctx, Command::HandOverPlayer));
        cmds
    }
}

impl CharStates {
    #[inline]
    pub fn relative_switch_char_idx(&self, active_char_idx: u8, switch_type: RelativeCharIdx) -> Option<u8> {
        switch_type
            .indexing_seq(active_char_idx, self.len())
            .find(|&j| self.is_valid_char_idx(j))
    }

    pub fn get_taken_most_dmg(&self) -> Option<(u8, &CharState)> {
        self.enumerate_valid().max_by_key(|(_, c)| c.get_total_dmg_taken())
    }
}
