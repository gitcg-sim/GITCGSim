use std::cmp::min;
use std::collections::VecDeque;

use smallvec::{smallvec, SmallVec};

use crate::data_structures::capped_list::CappedLengthList8;
use crate::data_structures::{CommandList, Vector};
use crate::dispatcher::cmd_trigger_event;
use crate::tcg_model::enums::*;

use super::exec_command_helpers::*;
use super::state_ops::check_valid_char_index;
use super::types::{DispatchResult, NondetRequest};
use crate::types::status_impl::RespondsTo;
use crate::types::status_impl::StatusImpl;
use crate::zobrist_hash::game_state_mutation::PlayerHashContext;
use crate::zobrist_hash::ZobristHasher;
use crate::{
    cards::ids::{lookup::*, *},
    dispatcher_ops::types::DispatchError,
    reaction::find_reaction,
    types::{card_defs::*, command::*, deal_dmg::*, dice_counter::DiceCounter, game_state::*, logging::Event},
};
use crate::{chc, cmd_list, mutate_statuses, mutate_statuses_1, phc, view};

impl GameState {
    /// Attempt to pay the cost. Succeeds without cost payment if `ignore_costs` is true.
    pub fn pay_cost(&mut self, cost: &Cost, cost_type: CostType) -> Result<(), DispatchError> {
        if self.ignore_costs {
            return Ok(());
        }

        let log = &mut self.log;
        if let Some(active_player_id) = self.phase.active_player() {
            let player = self.players.get_mut(active_player_id);
            let mut cost = *cost;
            augment_cost(phc!(self, active_player_id), player, &mut cost, cost_type);

            if cost.energy_cost > 0 {
                let ec = cost.energy_cost;
                let active_char_index = player.active_char_index;
                if let Some(active_char) = player.try_get_character_mut(active_char_index) {
                    let e = active_char.get_energy();
                    if e >= ec {
                        active_char.set_energy_hashed(chc!(self, active_player_id, active_char_index), e - ec);
                    } else {
                        return Err(DispatchError::UnableToPayCost);
                    }
                } else {
                    return Err(DispatchError::UnableToPayCost);
                }
            }

            log.log(Event::PayCost(active_player_id, cost, cost_type));
            if let Some(d) = try_pay_dice_cost(phc!(self, active_player_id), player, &cost, cost_type) {
                player.set_dice_after_paying_cast(phc!(self, active_player_id), &d);
            } else {
                return Err(DispatchError::UnableToPayCost);
            }
            Ok(())
        } else {
            Err(DispatchError::UnableToPayCost)
        }
    }

    pub fn check_switch_is_fast_action(&self, player_id: PlayerId, src_char_idx: u8) -> bool {
        let player = self.players.get(player_id);
        let mut res = false;
        let sc = &player.status_collection;
        if sc.responds_to(RespondsTo::SwitchIsFastAction) {
            sc.consume_statuses_immutable(
                CharacterIndexSelector::One(src_char_idx),
                |si| si.responds_to().contains(RespondsTo::SwitchIsFastAction),
                |es, _sk, si| si.switch_is_fast_action(es, &mut res),
            );
        }
        res
    }

    /// Consumes the usages of the relevant statuses
    pub fn try_switch_is_fast_action(&mut self, player_id: PlayerId, src_char_idx: u8) -> bool {
        // TODO consume 1st instance, priortizing character passive first
        let player = self.players.get(player_id);
        let mut res = false;
        if player.status_collection.responds_to(RespondsTo::SwitchIsFastAction) {
            mutate_statuses!(self, player_id, |sc| {
                sc.consume_statuses_first(
                    CharacterIndexSelector::One(src_char_idx),
                    |si| si.responds_to().contains(RespondsTo::SwitchIsFastAction),
                    |es, _sk, si| si.switch_is_fast_action(es, &mut res),
                )
            });
        }
        res
    }

    fn end_of_turn(&mut self) -> ExecResult {
        if let Phase::EndPhase {
            next_first_active_player: first_active_player,
        } = self.phase
        {
            for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
                mutate_statuses!(self, player_id, |sc| {
                    sc.consume_statuses(
                        CharacterIndexSelector::All,
                        |_| true,
                        |eff_state, status_key, _| {
                            let status = status_key.get_status();
                            eff_state.end_of_turn(status);

                            if eff_state.should_be_eliminated(status) {
                                Some(AppliedEffectResult::DeleteSelf)
                            } else {
                                None
                            }
                        },
                    );
                });
                {
                    let player = self.players.get_mut(player_id);
                    player.clear_flags_for_end_of_turn();
                    for (char_idx, char_state) in player.char_states.iter_mut().enumerate() {
                        let next_flags = char_state.flags & CharFlag::RETAIN;
                        char_state.set_flags_hashed(chc!(self, player_id, char_idx as u8), next_flags);
                    }
                }
            }

            self.round_number += 1;
            self.set_phase(Phase::new_roll_phase(first_active_player));
            self.log.log(Event::Phase(self.phase));
        } else {
            panic!("end_of_turn: Not at End Phase.");
        }
        ExecResult::Success
    }

    fn trigger_event(&mut self, ctx: &CommandContext, event_id: EventId) -> ExecResult {
        let src_player_id = ctx.src_player_id;
        let mut cmds = cmd_list![];
        let ctx_for_dmg = self.ctx_for_dmg(src_player_id, ctx.src);
        let src_player = self.players.get_mut(src_player_id);
        if src_player.status_collection.responds_to_trigger_event(event_id) {
            let src_player_state = &view!(src_player);
            let sicb = StatusImplContextBuilder::new(src_player_state, ctx, ());
            mutate_statuses_1!(phc!(self, src_player_id), src_player, |sc| {
                sc.consume_statuses(
                    CharacterIndexSelector::All,
                    |si| {
                        si.responds_to().contains(RespondsTo::TriggerEvent)
                            && si.responds_to_triggers().contains(event_id)
                    },
                    |es, status_key, si| {
                        let mut ectx = TriggerEventContext {
                            c: sicb.build(status_key, es),
                            event_id,
                            status_key,
                            ctx_for_dmg: &ctx_for_dmg,
                            out_cmds: &mut cmds,
                        };
                        si.trigger_event(&mut ectx)
                    },
                );
            })
        }
        if self.log.enabled && !cmds.is_empty() {
            self.log.log(Event::TriggerEvent(ctx.src, event_id));
        }
        ExecResult::AdditionalCmds(cmds)
    }

    fn trigger_xevent(&mut self, ctx: &CommandContext, xevt: XEvent) -> ExecResult {
        let mut cmds = cmd_list![];
        let src_player_id = ctx.src_player_id;
        for (src_player_id, xevt, src) in [
            (src_player_id, xevt, ctx.src),
            (src_player_id.opposite(), xevt, CommandSource::Event),
        ] {
            let ctx_for_dmg = self.ctx_for_dmg(src_player_id, src);
            let src_player = self.players.get_mut(src_player_id);
            let mask = xevt.mask(src_player_id);
            if src_player.status_collection.responds_to_events(mask) {
                let src_player_state = &view!(src_player);
                let sicb = StatusImplContextBuilder::new(src_player_state, ctx, ());
                mutate_statuses_1!(phc!(self, src_player_id), src_player, |sc| {
                    sc.consume_statuses(
                        CharacterIndexSelector::All,
                        |si| {
                            si.responds_to().contains(RespondsTo::TriggerXEvent)
                                && !(si.responds_to_events() & mask).is_empty()
                        },
                        |es, status_key, si| {
                            let mut ectx = TriggerEventContext {
                                c: sicb.build(status_key, es),
                                event_id: xevt,
                                status_key,
                                ctx_for_dmg: &ctx_for_dmg,
                                out_cmds: &mut cmds,
                            };
                            si.trigger_xevent(&mut ectx)
                        },
                    );
                })
            }
            // TODO log this
            // if self.log.enabled && !cmds.is_empty() {
            //     self.log.log(Event::TriggerEvent(ctx.src, event_id));
            // }
        }
        ExecResult::AdditionalCmds(cmds)
    }

    fn ctx_for_dmg(&self, src_player_id: PlayerId, src: CommandSource) -> CommandContext {
        let tgt_player = self.players.get(src_player_id.opposite());
        let opp = src_player_id.opposite();
        let cmd_tgt = {
            if tgt_player.is_valid_char_index(tgt_player.active_char_index) {
                Some(CommandTarget {
                    player_id: opp,
                    char_idx: tgt_player.active_char_index,
                })
            } else {
                None
            }
        };
        CommandContext::new(src_player_id, src, cmd_tgt)
    }

    fn do_switch_character(&mut self, ctx: &CommandContext, char_index: u8) -> ExecResult {
        let p = self.players.get_mut(ctx.src_player_id);
        let prev_char_idx = p.active_char_index;
        // Switching into self or invalid character does nothing
        if !p.switch_character(phc!(self, ctx.src_player_id), char_index) {
            return ExecResult::Success;
        }
        let sw = CommandSource::Switch {
            from_char_idx: prev_char_idx,
            dst_char_idx: char_index,
        };
        ExecResult::AdditionalCmds(cmd_list![(
            CommandContext::new(ctx.src_player_id, sw, None),
            Command::TriggerEvent(EventId::Switched)
        )])
    }

    fn switch_relative(&mut self, ctx: &CommandContext, switch_type: RelativeSwitchType) -> ExecResult {
        let Some(char_idx) = self.players[ctx.src_player_id].relative_switch_char_idx(switch_type) else {
            return ExecResult::Success
        };
        self.do_switch_character(ctx, char_idx)
    }

    fn apply_element_to_self(&mut self, ctx: &CommandContext, elem: Element) -> ExecResult {
        let char_idx = if let CommandSource::Skill { char_idx, .. } = ctx.src {
            char_idx
        } else {
            self.players.get(ctx.src_player_id).active_char_index
        };

        let src_player = self.players.get_mut(ctx.src_player_id);
        let Some(src_char) = src_player.try_get_character_mut(char_idx) else {
            return ExecResult::Success
        };

        let c = chc!(self, ctx.src_player_id, char_idx);
        if let (app1, Some((_, _))) = find_reaction(src_char.applied, elem) {
            src_char.set_applied_elements_hashed(c, app1);
        } else if elem.can_be_applied() {
            src_char.set_applied_elements_hashed(c, src_char.applied | elem);
        }
        ExecResult::Success
    }

    fn deal_dmg(&mut self, ctx: &CommandContext, dmg: DealDMG) -> ExecResult {
        let Some(CommandTarget { char_idx: tgt_char_idx, player_id: tgt_player_id }) = ctx.tgt else {
            panic!("deal_dmg: No dst_char for ctx");
        };

        let log = &mut self.log;
        if tgt_player_id != ctx.src_player_id.opposite() {
            panic!("deal_dmg: Invalid tgt_player_id");
        }

        let (src_player_id, tgt_player_id) = (ctx.src_player_id, ctx.src_player_id.opposite());
        let (src_player, tgt_player) = self.players.get_two_mut(src_player_id);
        if !tgt_player.is_valid_char_index(tgt_char_idx) {
            return ExecResult::Success;
        }

        // TODO shift statuses here
        let mut defeated = CharIdxSet::default();
        let mut addl_cmds: SmallVec<[(CommandContext, Command); 8]> = cmd_list![];
        let mut i = 0;
        let mut targets: SmallVec<[_; 4]> = smallvec![(tgt_char_idx, dmg)];
        while i < targets.len() {
            let (tgt_char_idx, mut dmg) = targets[i];
            let is_piercing = dmg.dmg_type == DealDMGType::Piercing;
            if !check_valid_char_index(&tgt_player.char_states, tgt_char_idx) {
                i += 1;
                continue;
            }

            let (tgt_applied, log_tgt, dmg_info) = {
                let tgt_char = &tgt_player.char_states[tgt_char_idx as usize];
                let tgt_applied = tgt_char.applied;
                let log_tgt = (tgt_char_idx, tgt_char.char_id);
                let dmg_info: DMGInfo = DMGInfo {
                    target_hp: tgt_char.get_hp(),
                    target_affected_by_riptide: tgt_player
                        .status_collection
                        .has_character_status(tgt_char_idx, StatusId::Riptide),
                };
                (tgt_applied, log_tgt, dmg_info)
            };

            macro_rules! apply_statuses {
                (($src_player: ident, $src_player_id: ident, $rto: expr, $dmg_info: expr), |$sc_src: ident, $sicb: ident| $expr: expr) => {
                    if $src_player.status_collection.responds_to($rto) {
                        let view = &view!($src_player);
                        let $sicb = StatusImplContextBuilder::new(view, ctx, $dmg_info);
                        mutate_statuses_1!(phc!(self, $src_player_id), $src_player, |$sc_src| { $expr });
                    }
                };
            }

            if !is_piercing {
                apply_statuses!(
                    (src_player, src_player_id, RespondsTo::OutgoingDMG, dmg_info),
                    |sc_src, sicb| augment_outgoing_dmg_for_statuses(sc_src, sicb, &mut dmg)
                );
                apply_statuses!(
                    (src_player, src_player_id, RespondsTo::LateOutgoingDMG, dmg_info),
                    |sc_src, sicb| augment_late_outgoing_dmg_for_statuses(sc_src, sicb, &mut dmg)
                );
            }

            // Resolve Elemental Reaction
            let mut reaction: Option<(Reaction, Option<Element>)> = None;
            let (new_tgt_applied, rxn_cmd, log_rxn) = match dmg.dmg_type {
                DealDMGType::Piercing => (tgt_applied, None, None),
                DealDMGType::Physical => (tgt_applied, None, None),
                DealDMGType::Elemental(e) => {
                    if let (app1, Some((rxn, te))) = find_reaction(tgt_applied, e) {
                        if i == 0 {
                            reaction = Some((rxn, te));
                        }
                        let (dmg_bonus, piercing, rxn_cmd) = rxn.reaction_effects(te);
                        dmg.dmg += dmg_bonus;
                        dmg.piercing_dmg_to_standby += piercing;

                        (app1, rxn_cmd, Some(Event::Reaction(tgt_player_id, log_tgt, rxn)))
                    } else if e.can_be_applied() {
                        (
                            tgt_applied | e,
                            None,
                            Some(Event::ElemApplied(tgt_player_id, log_tgt, e)),
                        )
                    } else {
                        (tgt_applied, None, None)
                    }
                }
            };
            {
                let tgt_char = &mut tgt_player.char_states[tgt_char_idx as usize];
                let c = chc!(self, tgt_player_id, tgt_char_idx);
                tgt_char.set_applied_elements_hashed(c, new_tgt_applied);
            }

            if let Some(reaction) = reaction {
                apply_statuses!(
                    (src_player, src_player_id, RespondsTo::OutgoingReactionDMG, dmg_info),
                    |sc_src, sicb| augment_outgoing_reaction_dmg_for_statuses(sc_src, sicb, reaction, &mut dmg)
                );
            }

            if !is_piercing {
                let mut mult = 1;
                apply_statuses!(
                    (src_player, src_player_id, RespondsTo::MultiplyOutgoingDMG, dmg_info),
                    |sc_src, sicb| multiply_outgoing_dmg_for_statuses(sc_src, sicb, &mut mult)
                );
                dmg.dmg *= mult;

                if tgt_player.status_collection.has_shield_points() {
                    mutate_statuses_1!(phc!(self, tgt_player_id), tgt_player, |sc_tgt| {
                        consume_shield_points_for_statuses(sc_tgt, tgt_char_idx, &mut dmg);
                    });
                }

                apply_statuses!(
                    (tgt_player, tgt_player_id, RespondsTo::IncomingDMG, ()),
                    |sc_tgt, sicb| augment_incoming_dmg_for_statuses(sc_tgt, sicb, tgt_char_idx, &mut dmg)
                );
            }

            let mut reduce_hp = |tgt_char: &mut CharState, tgt_char_idx: u8, dmg_value: u8| -> bool {
                tgt_char.reduce_hp_hashed(chc!(self, tgt_player_id, tgt_char_idx), dmg_value);
                if !tgt_char.is_invalid() {
                    return false;
                }

                if let Ok(tgt_char_idx) = tgt_char_idx.try_into() {
                    defeated.insert(tgt_char_idx);
                    true
                } else {
                    false
                }
            };

            {
                let tgt_char = &mut tgt_player.char_states[tgt_char_idx as usize];
                let defeated = reduce_hp(tgt_char, tgt_char_idx, dmg.dmg);
                addl_cmds.push((
                    *ctx,
                    Command::TriggerXEvent(XEvent::DMG(XEventDMG {
                        src_player_id,
                        tgt_char_idx,
                        dmg_value: dmg.dmg,
                        dmg_type: dmg.dmg_type,
                        dmg_info,
                        reaction,
                        defeated,
                    })),
                ));
            }

            if log.enabled {
                // TODO support DMGSource
                let dmg_source = None;
                log.log(Event::DealDMG(dmg_source, (tgt_player_id, log_tgt), dmg));
                if let Some(e) = log_rxn {
                    log.log(e);
                }
            }

            if dmg.piercing_dmg_to_standby > 0 {
                let new_char_states: &mut Vector<CharState> = &mut tgt_player.char_states;

                let pd = dmg.piercing_dmg_to_standby;
                for j in 0..new_char_states.len() {
                    if tgt_char_idx == (j as u8) || new_char_states[j].is_invalid() {
                        continue;
                    }

                    let pdmg = DealDMG::new(DealDMGType::Physical, pd, 0);
                    {
                        let defeated = reduce_hp(&mut new_char_states[j], j as u8, pdmg.dmg);
                        addl_cmds.push((
                            *ctx,
                            Command::TriggerXEvent(XEvent::DMG(XEventDMG {
                                src_player_id,
                                tgt_char_idx,
                                dmg_value: dmg.dmg,
                                dmg_type: DealDMGType::Piercing,
                                dmg_info,
                                reaction: None,
                                defeated,
                            })),
                        ));
                    }
                }
            }

            match rxn_cmd {
                Some(Command::DealSwirlDMG(e, ..)) => {
                    for j in 0..tgt_player.char_states.len() {
                        let j = j as u8;
                        if j == tgt_char_idx || tgt_player.char_states[i].is_invalid() {
                            continue;
                        }
                        targets.push((
                            j,
                            DealDMG {
                                dmg_type: DealDMGType::Elemental(e),
                                dmg: 1,
                                piercing_dmg_to_standby: 0,
                            },
                        ))
                    }
                }
                Some(cmd) => {
                    let new_ctx = ctx.with_tgt(Some(CommandTarget::new(tgt_player_id, tgt_char_idx)));
                    addl_cmds.push((new_ctx, cmd));
                }
                _ => (),
            }
            i += 1;
        }

        self.resolve_defeated(tgt_player_id, ctx, defeated, &mut addl_cmds);
        ExecResult::AdditionalCmds(addl_cmds)
    }

    fn resolve_defeated(
        &mut self,
        player_id: PlayerId,
        ctx: &CommandContext,
        defeated: CharIdxSet,
        addl_cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        if defeated.is_empty() {
            return;
        }

        let player = self.players.get_mut(player_id);
        let mut shifts_to_next_active: SmallVec<[(StatusId, AppliedEffectState); 2]> = Default::default();

        for char_idx in defeated {
            let char_idx: u8 = char_idx.into();
            let char_state = &mut player.char_states[char_idx as usize];
            char_state.set_energy_hashed(chc!(self, player_id, char_idx), 0);
            char_state.set_applied_elements_hashed(chc!(self, player_id, char_idx), Default::default());
            char_state.set_flags_hashed(chc!(self, player_id, char_idx), Default::default());

            mutate_statuses_1!(phc!(self, player_id), player, |sc| {
                sc.clear_character_statuses(char_idx, &mut shifts_to_next_active);
            });

            let log = &mut self.log;
            if log.enabled {
                log.log(Event::CharacterDied(
                    player_id,
                    (char_idx, player.get_active_character().char_id),
                ));
            }
        }

        for (status_id, eff_state) in shifts_to_next_active {
            addl_cmds.push((
                *ctx,
                Command::ApplyCharacterStatusToActive(player_id, status_id, eff_state),
            ));
        }
    }

    fn take_dmg(&mut self, ctx: &CommandContext, dmg: DealDMG) -> ExecResult {
        let Some(char_idx) = ctx.src.char_idx() else {
            return ExecResult::Success
        };
        let ctx = CommandContext::new(
            ctx.src_player_id.opposite(),
            ctx.src,
            Some(CommandTarget {
                player_id: ctx.src_player_id,
                char_idx,
            }),
        );
        self.deal_dmg(&ctx, dmg)
    }

    fn take_dmg_for_affected_by(&mut self, ctx: &CommandContext, status_id: StatusId, dmg: DealDMG) -> ExecResult {
        let player = self.players.get_mut(ctx.src_player_id);
        let sc = &player.status_collection;
        let mut cmds = smallvec![];
        for (i, c) in player.char_states.iter().enumerate() {
            if c.is_invalid() {
                continue;
            }
            let char_idx = i as u8;
            if !sc.has_character_status(char_idx, status_id) {
                continue;
            }

            let ctx = CommandContext::new(ctx.src_player_id, CommandSource::Character { char_idx }, None);
            cmds.push((ctx, Command::TakeDMG(dmg)));
        }
        ExecResult::AdditionalCmds(cmds)
    }

    fn add_energy(&mut self, ctx: &CommandContext, energy: u8) -> ExecResult {
        let p = self.players.get_mut(ctx.src_player_id);
        let char_idx = ctx.src.selected_char_index_or(p.active_char_index);
        self.add_energy_to_character(ctx, energy, char_idx)
    }

    fn add_energy_without_maximum(&mut self, ctx: &CommandContext, energy: u8) -> ExecResult {
        let check = |c: &CharState| !c.is_invalid() && c.get_energy() < c.char_id.get_char_card().max_energy;
        let char_idx = {
            let p = self.players.get(ctx.src_player_id);
            let active_char = p.get_active_character();
            if check(active_char) {
                Some(p.active_char_index)
            } else {
                p.char_states
                    .iter()
                    .enumerate()
                    .find(|(_, c)| check(c))
                    .map(|(i, _)| i as u8)
            }
        };

        if let Some(char_idx) = char_idx {
            self.add_energy_to_character(ctx, energy, char_idx)
        } else {
            ExecResult::Success
        }
    }

    fn add_energy_to_character(&mut self, ctx: &CommandContext, energy: u8, char_idx: u8) -> ExecResult {
        let p = self.players.get_mut(ctx.src_player_id);
        if let Some(active_char) = p.try_get_character_mut(char_idx) {
            active_char.add_energy_hashed(chc!(self, ctx.src_player_id, char_idx), energy);
        }
        ExecResult::Success
    }

    fn add_energy_to_non_active_characters(&mut self, ctx: &CommandContext, energy: u8) -> ExecResult {
        let player = self.players.get_mut(ctx.src_player_id);
        let active_char_idx = player.active_char_index;
        for (i, char_state) in player.char_states.iter_mut().enumerate() {
            let char_idx = i as u8;
            if char_idx == active_char_idx || char_state.is_invalid() {
                continue;
            }
            char_state.add_energy_hashed(chc!(self, ctx.src_player_id, char_idx), energy);
        }
        ExecResult::Success
    }

    fn set_energy(&mut self, ctx: &CommandContext, energy: u8) -> ExecResult {
        let p = self.players.get_mut(ctx.src_player_id);
        let char_idx = ctx.src.selected_char_index_or(p.active_char_index);
        if let Some(active_char) = p.try_get_character_mut(char_idx) {
            active_char.set_energy_hashed(chc!(self, ctx.src_player_id, char_idx), energy);
        }
        ExecResult::Success
    }

    fn shift_energy(&mut self, ctx: &CommandContext) -> ExecResult {
        let player = self.players.get_mut(ctx.src_player_id);
        let char_idx = ctx.src.selected_char_index_or(player.active_char_index);
        let mut total = 0;
        for (i, char_state) in player.char_states.iter_mut().enumerate() {
            let i = i as u8;
            if i == char_idx || char_state.get_energy() == 0 {
                continue;
            }
            char_state.set_energy_hashed(chc!(self, ctx.src_player_id, i), char_state.get_energy() - 1);
            total += 1;
            if total >= 2 {
                break;
            }
        }
        let char_state = &mut player.char_states[char_idx as usize];
        let new_energy = min(
            char_state.get_energy() + total,
            char_state.char_id.get_char_card().max_energy,
        );
        char_state.set_energy_hashed(chc!(self, ctx.src_player_id, char_idx), new_energy);
        ExecResult::Success
    }

    fn increase_status_usages(&mut self, ctx: &CommandContext, key: StatusKey, usages: u8) -> ExecResult {
        'a: {
            mutate_statuses!(self, ctx.src_player_id, |sc| {
                let Some(eff_state) = sc.get_mut(key) else { break 'a };

                let status = key.get_status();
                if status.duration_rounds.is_some() {
                    eff_state.set_duration(eff_state.get_duration() + usages);
                } else if status.usages.is_some() {
                    eff_state.set_usages(eff_state.get_usages() + usages);
                } else {
                    panic!(
                        "increase_status_usages: Does not have a Usages/Duration counter: {:?}",
                        key
                    )
                }
            })
        };
        ExecResult::Success
    }

    fn delete_status(&mut self, ctx: &CommandContext, key: StatusKey) -> ExecResult {
        mutate_statuses!(self, ctx.src_player_id, |sc| { sc.delete(key) });
        ExecResult::Success
    }

    fn delete_status_for_target(&mut self, ctx: &CommandContext, key: StatusKey) -> ExecResult {
        mutate_statuses!(self, ctx.src_player_id.opposite(), |sc| { sc.delete(key) });
        ExecResult::Success
    }

    fn reroll_dice(&mut self, _ctx: &CommandContext) -> ExecResult {
        // TODO implement reroll existing, newly-added, dice
        ExecResult::Success
    }

    fn heal(&mut self, ctx: &CommandContext, hp: u8) -> ExecResult {
        let p = self.players.get_mut(ctx.src_player_id);
        let char_idx = ctx.src.selected_char_index_or(p.active_char_index);
        if let Some(active_char) = p.try_get_character_mut(char_idx) {
            active_char.heal_hashed(chc!(self, ctx.src_player_id, char_idx), hp);
            if self.log.enabled {
                self.log
                    .log(Event::Heal(ctx.src_player_id, (char_idx, active_char.char_id), hp));
            }
        }
        ExecResult::Success
    }

    fn heal_all(&mut self, ctx: &CommandContext, hp: u8) -> ExecResult {
        let p = self.players.get_mut(ctx.src_player_id);
        for char_idx in 0..(p.char_states.len() as u8) {
            let Some(character) = p.try_get_character_mut(char_idx) else {
                continue
            };
            character.heal(hp);
            if self.log.enabled {
                self.log
                    .log(Event::Heal(ctx.src_player_id, (char_idx, character.char_id), hp));
            }
        }
        ExecResult::Success
    }

    fn add_dice(&mut self, ctx: &CommandContext, dice: &DiceCounter) -> ExecResult {
        let player_id = ctx.src_player_id;
        self.players.get_mut(player_id).add_dice(phc!(self, player_id), dice);
        ExecResult::Success
    }

    fn subtract_dice(&mut self, ctx: &CommandContext, dice: &DiceCounter) -> ExecResult {
        let player_id = ctx.src_player_id;
        self.players
            .get_mut(player_id)
            .subtract_dice(phc!(self, player_id), dice);
        ExecResult::Success
    }

    fn add_cards_to_hand(&mut self, ctx: &CommandContext, cards: &CappedLengthList8<CardId>) -> ExecResult {
        let player_id = ctx.src_player_id;
        let player = self.players.get_mut(player_id);
        for card_id in cards.to_vec() {
            player.add_card_to_hand(phc!(self, player_id), card_id);
        }

        if self.tactical {
            player.pseudo_elemental_tuning(phc!(self, player_id));
        }

        ExecResult::Success
    }

    fn draw_cards(&mut self, ctx: &CommandContext, count: u8, card_type: Option<CardType>) -> ExecResult {
        ExecResult::Suspend(
            SuspendedState::NondetRequest(NondetRequest::DrawCardsOfType(ctx.src_player_id, count, card_type)),
            None,
        )
    }

    #[inline]
    fn apply_or_refresh_status(&mut self, src_player_id: PlayerId, key: StatusKey, status: &'static Status) {
        let src_player = self.players.get_mut(src_player_id);
        let modifiers = src_player.get_status_spec_modifiers(key);
        mutate_statuses_1!(phc!(self, src_player_id), src_player, |sc| {
            sc.apply_or_refresh_status(key, status, &modifiers);
        });
    }

    fn hand_over_player(&mut self) -> ExecResult {
        let (first_end_round, next_player) = match self.phase {
            Phase::RollPhase {
                first_active_player,
                roll_phase_state: RollPhaseState::Rolling,
            } => (None, first_active_player),
            Phase::ActionPhase {
                first_end_round: None,
                active_player,
            } => (None, active_player.opposite()),
            Phase::ActionPhase {
                first_end_round: Some(first_end_round),
                active_player,
            } => (Some(first_end_round), active_player),
            _ => return ExecResult::Success,
        };
        self.set_phase(Phase::ActionPhase {
            first_end_round,
            active_player: next_player,
        });
        ExecResult::AdditionalCmds(cmd_list![cmd_trigger_event(next_player, EventId::BeforeAction)])
    }

    fn apply_status_to_team(&mut self, ctx: &CommandContext, status_id: StatusId) -> ExecResult {
        self.log.log(Event::ApplyTeamStatus(ctx.src_player_id, status_id));
        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Team {
            panic!("apply_status_to_team: wrong StatusAttachMode");
        }

        self.apply_or_refresh_status(ctx.src_player_id, StatusKey::Team(status_id), status);
        ExecResult::Success
    }

    fn apply_status_to_target(&mut self, ctx: &CommandContext, status_id: StatusId) -> ExecResult {
        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Character {
            panic!("apply_status_to_target: wrong StatusAttachMode");
        }

        if !status.applies_to_opposing {
            panic!("apply_status_to_target: applies_to_opposing is false");
        }

        let Some(tgt_player_id) = ctx.tgt.map(|t| t.player_id) else {
            panic!("apply_status_to_target: no target");
        };
        let tgt_player = self.players.get_mut(tgt_player_id);
        let tgt_char_idx = ctx.tgt.map(|x| x.char_idx).unwrap_or(tgt_player.active_char_index);
        if !tgt_player.is_valid_char_index(tgt_char_idx) {
            return ExecResult::Success;
        }

        self.log.log(Event::ApplyCharStatus(
            tgt_player_id,
            (tgt_char_idx, tgt_player.get_active_character().char_id),
            status_id,
        ));

        self.apply_or_refresh_status(tgt_player_id, StatusKey::Character(tgt_char_idx, status_id), status);
        ExecResult::Success
    }

    fn apply_status_to_target_team(&mut self, ctx: &CommandContext, status_id: StatusId) -> ExecResult {
        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Team {
            panic!("apply_status_to_target_team: wrong StatusAttachMode");
        }
        if !status.applies_to_opposing {
            panic!("apply_status_to_target_team: applies_to_opposing is false");
        }
        let Some(tgt_player_id) = ctx.tgt.map(|t| t.player_id) else {
            panic!("apply_status_to_target_team: no target player");
        };

        self.log.log(Event::ApplyTeamStatus(tgt_player_id, status_id));
        self.apply_or_refresh_status(tgt_player_id, StatusKey::Team(status_id), status);
        ExecResult::Success
    }

    fn apply_status_to_opponent_characters(&mut self, ctx: &CommandContext, status_id: StatusId) -> ExecResult {
        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Character {
            panic!("apply_status_to_opponent_characters: wrong StatusAttachMode");
        }
        if !status.applies_to_opposing {
            panic!("apply_status_to_opponent_characters: applies_to_opposing is false");
        }
        let Some(tgt_player_id) = ctx.tgt.map(|t| t.player_id) else {
            panic!("apply_status_to_target: no target");
        };

        let tgt_char_states = &self.players[tgt_player_id].char_states;
        let to_apply: Vector<(u8, CharId)> = tgt_char_states
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_invalid())
            .map(|(i, c)| (i as u8, c.char_id))
            .collect();
        for (tgt_char_idx, char_id) in to_apply {
            self.log.log(Event::ApplyCharStatus(
                tgt_player_id,
                (tgt_char_idx, char_id),
                status_id,
            ));

            self.apply_or_refresh_status(tgt_player_id, StatusKey::Character(tgt_char_idx, status_id), status);
        }
        ExecResult::Success
    }

    fn apply_status_to_character(&mut self, ctx: &CommandContext, status_id: StatusId, char_idx: u8) -> ExecResult {
        let player = self.players.get_mut(ctx.src_player_id);
        if !player.is_valid_char_index(char_idx) {
            return ExecResult::Success;
        }

        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Character {
            panic!("apply_status_to_target_team: wrong StatusAttachMode");
        }
        self.apply_or_refresh_status(ctx.src_player_id, StatusKey::Character(char_idx, status_id), status);

        let log = &mut self.log;
        if log.enabled {
            let player = self.players.get_mut(ctx.src_player_id);
            log.log(Event::ApplyCharStatus(
                ctx.src_player_id,
                (char_idx, player.try_get_character(char_idx).unwrap().char_id),
                status_id,
            ));
        }
        ExecResult::Success
    }

    fn apply_status_to_active_character(&mut self, ctx: &CommandContext, status_id: StatusId) -> ExecResult {
        let active_player = self
            .get_active_player()
            .unwrap_or_else(|| self.players.get(ctx.src_player_id));
        self.apply_status_to_character(ctx, status_id, active_player.active_char_index)
    }

    fn apply_equipment_to_character(
        &mut self,
        ctx: &CommandContext,
        slot: EquipSlot,
        status_id: StatusId,
        char_idx: u8,
    ) -> ExecResult {
        let player = self.players.get_mut(ctx.src_player_id);
        if !player.is_valid_char_index(char_idx) {
            return ExecResult::Success;
        }

        let status = status_id.get_status();
        mutate_statuses_1!(phc!(self, ctx.src_player_id), player, |sc| {
            sc.ensure_unequipped(char_idx, slot);
            // Equipment usages cannot be buffed
            sc.apply_or_refresh_status(StatusKey::Equipment(char_idx, slot, status_id), status, &None);
        });
        let log = &mut self.log;
        if log.enabled {
            log.log(Event::Equip(
                ctx.src_player_id,
                (char_idx, player.try_get_character(char_idx).unwrap().char_id),
                slot,
                Some(status_id),
            ));
        }
        ExecResult::Success
    }

    fn apply_talent_to_character(
        &mut self,
        ctx: &CommandContext,
        char_idx: u8,
        status_id: Option<StatusId>,
    ) -> ExecResult {
        let player = self.players.get_mut(ctx.src_player_id);
        if !player.is_valid_char_index(char_idx) {
            return ExecResult::Success;
        }
        let char_state = &mut player.char_states[char_idx as usize];
        let flags = char_state.flags | CharFlag::TalentEquipped;
        if let Some(status_id) = status_id {
            let slot = EquipSlot::Talent;
            let status = status_id.get_status();
            mutate_statuses_1!(phc!(self, ctx.src_player_id), player, |sc| {
                sc.ensure_unequipped(char_idx, slot);
                sc.apply_or_refresh_status(StatusKey::Equipment(char_idx, slot, status_id), status, &None);
            });
        }

        char_state.set_flags_hashed(chc!(self, ctx.src_player_id, char_idx), flags);
        let log = &mut self.log;
        if log.enabled {
            log.log(Event::Equip(
                ctx.src_player_id,
                (char_idx, player.try_get_character(char_idx).unwrap().char_id),
                EquipSlot::Talent,
                status_id,
            ));
        }
        ExecResult::Success
    }

    fn apply_character_status_to_active(
        &mut self,
        player_id: PlayerId,
        status_id: StatusId,
        eff_state: AppliedEffectState,
    ) -> ExecResult {
        let player = self.players.get_mut(player_id);
        player
            .status_collection
            .set_status(StatusKey::Character(player.active_char_index, status_id), eff_state);
        ExecResult::Success
    }

    fn add_support(&mut self, ctx: &CommandContext, slot: SupportSlot, support_id: SupportId) -> ExecResult {
        let player_id = ctx.src_player_id;
        let player = self.players.get_mut(player_id);
        mutate_statuses_1!(phc!(self, ctx.src_player_id), player, |sc| {
            sc.add_support_to_slot_replacing_existing(slot, support_id);
        });
        ExecResult::Success
    }

    fn force_switch_for_target(&mut self, ctx: &CommandContext, force_switch_type: RelativeSwitchType) -> ExecResult {
        let Some(tgt_player_id) = ctx.tgt.map(|t| t.player_id) else {
            panic!("force_switch_for_target: no target");
        };
        let tgt_player = self.players.get_mut(tgt_player_id);
        let tgt_char_idx = ctx.tgt.map(|x| x.char_idx).unwrap_or(tgt_player.active_char_index);

        if tgt_player.active_char_index != tgt_char_idx {
            return ExecResult::Success;
        }

        let Some(tgt_char_idx) = tgt_player.relative_switch_char_idx(force_switch_type) else {
            return ExecResult::Success;
        };

        let ctx1 = CommandContext::new(tgt_player_id, ctx.src, None);
        self.do_switch_character(&ctx1, tgt_char_idx)
    }

    fn summon(&mut self, ctx: &CommandContext, summon_id: SummonId) -> ExecResult {
        self.log.log(Event::Summon(ctx.src_player_id, summon_id));
        let status = summon_id.get_status();
        self.apply_or_refresh_status(ctx.src_player_id, StatusKey::Summon(summon_id), status);
        ExecResult::Success
    }

    fn summon_random(&mut self, _: &CommandContext, spec: SummonRandomSpec) -> ExecResult {
        ExecResult::Suspend(SuspendedState::NondetRequest(NondetRequest::SummonRandom(spec)), None)
    }

    fn post_death_check(&mut self, prev_res: ExecResult) -> ExecResult {
        for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
            let player = self.players.get_mut(player_id);
            if player.is_valid_char_index(player.active_char_index) {
                continue;
            }

            // Found a winner due to all characters being dead
            if player.char_states.iter().all(|s| s.is_invalid()) {
                return ExecResult::Return(DispatchResult::Winner(player_id.opposite()));
            }

            // Ask the player to switch character after death
            let prev_addl_cmds = match prev_res {
                ExecResult::AdditionalCmds(ac) => Some(ac),
                _ => None,
            };

            player.flags.insert(PlayerFlags::DiedThisRound);
            return ExecResult::Suspend(SuspendedState::post_death_switch(player_id), prev_addl_cmds);
        }

        prev_res
    }

    fn stellar_restoration_from_skill(&mut self, ctx: &CommandContext) -> ExecResult {
        let player_id = ctx.src_player_id;
        let active_player = self.players.get_mut(player_id);
        let char_idx = active_player.active_char_index;
        let mut h = ZobristHasher::new();
        let res = if active_player.try_remove_card_from_hand((&mut h, player_id), CardId::LightningStiletto) {
            ExecResult::AdditionalCmds(cmd_list![(
                *ctx,
                Command::ApplyStatusToCharacter(StatusId::ElectroInfusion, char_idx)
            )])
        } else {
            if !matches!(ctx.src, CommandSource::Card { .. }) {
                active_player.add_card_to_hand((&mut h, player_id), CardId::LightningStiletto);
                if self.tactical {
                    active_player.pseudo_elemental_tuning((&mut h, player_id));
                }
            }
            ExecResult::Success
        };
        self._incremental_hash.combine(h);
        res
    }

    fn cast_skill_from_cmd(&mut self, ctx: &CommandContext, skill_id: SkillId) -> ExecResult {
        let player_id = ctx.src_player_id;
        let player = self.players.get_mut(player_id);
        let char_idx = ctx.src.char_idx().unwrap_or(player.active_char_index);
        let chc = chc!(self, player_id, char_idx);
        {
            let char = &mut player.char_states[char_idx as usize];
            let flags = char.flags | char.skill_flags(skill_id);
            char.set_flags_hashed(chc, flags);
        }
        let cmds = get_cast_skill_cmds(player, ctx, skill_id);
        ExecResult::AdditionalCmds(cmds)
    }

    fn exec(&mut self, ctx: &CommandContext, cmd: Command) -> ExecResult {
        let res: ExecResult = match cmd {
            Command::Nop => ExecResult::Success,
            Command::CastSkill(skill_id) => self.cast_skill_from_cmd(ctx, skill_id),
            Command::TriggerEvent(evt) => self.trigger_event(ctx, evt),
            Command::TriggerXEvent(xevt) => self.trigger_xevent(ctx, xevt),
            Command::SwitchCharacter(idx) => self.do_switch_character(ctx, idx),
            Command::ApplyElementToSelf(e) => self.apply_element_to_self(ctx, e),
            Command::DealDMG(d) => self.deal_dmg(ctx, d),
            Command::TakeDMG(d) => self.take_dmg(ctx, d),
            Command::TakeDMGForAffectedBy(status_id, d) => self.take_dmg_for_affected_by(ctx, status_id, d),
            Command::DealSwirlDMG(_, _) => panic!("Cannot execute DealSwirlDMG command."),
            Command::Heal(v) => self.heal(ctx, v),
            Command::HealAll(v) => self.heal_all(ctx, v),
            Command::AddEnergy(v) => self.add_energy(ctx, v),
            Command::AddEnergyWithoutMaximum(v) => self.add_energy_without_maximum(ctx, v),
            Command::AddEnergyToCharacter(v, i) => self.add_energy_to_character(ctx, v, i),
            Command::AddEnergyToNonActiveCharacters(v) => self.add_energy_to_non_active_characters(ctx, v),
            Command::SetEnergy(v) => self.set_energy(ctx, v),
            Command::ShiftEnergy => self.shift_energy(ctx),
            Command::IncreaseStatusUsages(key, usages) => self.increase_status_usages(ctx, key, usages),
            Command::DeleteStatus(key) => self.delete_status(ctx, key),
            Command::DeleteStatusForTarget(key) => self.delete_status_for_target(ctx, key),
            Command::RerollDice => self.reroll_dice(ctx),
            Command::AddDice(d) => self.add_dice(ctx, &d),
            Command::SubtractDice(d) => self.subtract_dice(ctx, &d),
            Command::AddCardsToHand(cards) => self.add_cards_to_hand(ctx, &cards),
            Command::DrawCards(n, t) => self.draw_cards(ctx, n, t),
            Command::ApplyStatusToActiveCharacter(status_id) => self.apply_status_to_active_character(ctx, status_id),
            Command::ApplyStatusToCharacter(status_id, char_idx) => {
                self.apply_status_to_character(ctx, status_id, char_idx)
            }
            Command::ApplyEquipmentToCharacter(slot, status_id, char_idx) => {
                self.apply_equipment_to_character(ctx, slot, status_id, char_idx)
            }
            Command::ApplyTalentToCharacter(char_idx, status_id) => {
                self.apply_talent_to_character(ctx, char_idx, status_id)
            }
            Command::ApplyCharacterStatusToActive(player_id, status_id, eff_state) => {
                self.apply_character_status_to_active(player_id, status_id, eff_state)
            }
            Command::AddSupport(slot, support_id) => self.add_support(ctx, slot, support_id),
            Command::ApplyStatusToTeam(status_id) => self.apply_status_to_team(ctx, status_id),
            Command::ApplyStatusToTarget(status_id) => self.apply_status_to_target(ctx, status_id),
            Command::ApplyStatusToTargetTeam(status_id) => self.apply_status_to_target_team(ctx, status_id),
            Command::ApplyStatusToAllOpponentCharacters(status_id) => {
                self.apply_status_to_opponent_characters(ctx, status_id)
            }
            Command::Summon(summon_id) => self.summon(ctx, summon_id),
            Command::SummonRandom(spec) => self.summon_random(ctx, spec),
            Command::SwitchPrev => self.switch_relative(ctx, RelativeSwitchType::Previous),
            Command::SwitchNext => self.switch_relative(ctx, RelativeSwitchType::Next),
            Command::SwitchPrevForTarget => self.force_switch_for_target(ctx, RelativeSwitchType::Previous),
            Command::SwitchNextForTarget => self.force_switch_for_target(ctx, RelativeSwitchType::Next),
            Command::HandOverPlayer => self.hand_over_player(),
            Command::EndOfTurn => self.end_of_turn(),
            Command::StellarRestorationFromSkill => self.stellar_restoration_from_skill(ctx),
        };
        self.post_death_check(res)
    }

    fn suspend(
        &mut self,
        suspended_state: SuspendedState,
        pending_cmds: CommandList<(CommandContext, Command)>,
    ) -> Result<Option<DispatchResult>, DispatchError> {
        self.pending_cmds = Some(Box::new(PendingCommands {
            suspended_state,
            pending_cmds,
        }));
        Ok(Some(suspended_state.get_dispatch_result()))
    }

    /// Execute commands on the game state.
    ///
    /// Panics: For command precondition violations that requires the card or effect code to be fixed.
    /// Returns:
    ///  - `Err(..)` if the error is attributed to the input passed to the dispatcher.
    ///  - `Ok(Some(..))` if the winner has been decided
    ///  - `Ok(None)` otherwise
    pub fn exec_commands(
        &mut self,
        cmds: &CommandList<(CommandContext, Command)>,
    ) -> Result<Option<DispatchResult>, DispatchError> {
        let mut queue = VecDeque::new();
        queue.reserve(cmds.len());
        for c in cmds {
            queue.push_back(*c);
        }
        while !queue.is_empty() {
            let (ctx, cmd) = queue.pop_front().unwrap();
            match self.exec(&ctx, cmd) {
                ExecResult::Return(winner) => return Ok(Some(winner)),
                ExecResult::Suspend(ss, cmds) => {
                    if let Some(cmds) = cmds {
                        if !cmds.is_empty() {
                            let mut j = cmds.len();
                            while j > 0 {
                                queue.push_front(cmds[j - 1]);
                                j -= 1;
                            }
                        }
                    }

                    let mut pending_cmds_vec = cmd_list![];
                    for v in queue {
                        pending_cmds_vec.push(v);
                    }
                    return self.suspend(ss, pending_cmds_vec);
                }
                ExecResult::Success => {}
                ExecResult::AdditionalCmds(cmds) => {
                    if !cmds.is_empty() {
                        let mut j = cmds.len();
                        while j > 0 {
                            queue.push_front(cmds[j - 1]);
                            j -= 1;
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    pub fn handle_post_exec(&mut self, opt: Option<DispatchResult>) -> DispatchResult {
        if let Some(r) = opt {
            if let DispatchResult::Winner(winner) = r {
                self.set_phase(Phase::WinnerDecided { winner });
            }
            return r;
        }
        self.players.0.check_for_charged_attack();
        self.players.1.check_for_charged_attack();

        if let Some(active_player) = self.phase.active_player() {
            DispatchResult::PlayerInput(active_player)
        } else {
            DispatchResult::NoInput
        }
    }
}
