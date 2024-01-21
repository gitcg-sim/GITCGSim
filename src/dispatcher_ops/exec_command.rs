use crate::std_subset::{cmp::min, collections::VecDeque, Box};

use smallvec::{smallvec, SmallVec};

use crate::{
    cards::ids::{lookup::*, *},
    chc, cmd_list,
    data_structures::{capped_list::CappedLengthList8, CommandList, Vector},
    dispatcher::cmd_trigger_event,
    dispatcher_ops::{DispatchError, DispatchResult, ExecResult, NondetRequest},
    mutate_statuses, mutate_statuses_1, phc,
    reaction::find_reaction,
    types::{card_defs::*, command::*, dice_counter::*, game_state::*, logging::Event, status_impl::*, tcg_model::*},
    view,
    zobrist_hash::ZobristHasher,
};

use super::exec_command_helpers::*;

impl GameState {
    /// Attempt to pay the cost. Succeeds without cost payment if `ignore_costs` is true.
    pub fn pay_cost(&mut self, cost: &Cost, cost_type: CostType) -> Result<(), DispatchError> {
        if self.ignore_costs {
            return Ok(());
        }

        let log = &mut self.log;
        let Some(active_player_id) = self.phase.active_player() else {
            return Err(DispatchError::UnableToPayCost);
        };
        let player = self.players.get_mut(active_player_id);
        let mut cost = *cost;
        augment_cost(phc!(self, active_player_id), player, &mut cost, cost_type);

        if cost.energy_cost > 0 {
            let ec = cost.energy_cost;
            let active_char_idx = player.active_char_idx;
            let Some(active_char) = player.try_get_character_mut(active_char_idx) else {
                return Err(DispatchError::UnableToPayCost);
            };
            let e = active_char.get_energy();
            if e < ec {
                return Err(DispatchError::UnableToPayCost);
            }

            active_char.set_energy_hashed(chc!(self, active_player_id, active_char_idx), e - ec);
        }

        log.log(Event::PayCost(active_player_id, cost, cost_type));
        let Some(d) = try_pay_dice_cost(phc!(self, active_player_id), player, &cost, cost_type) else {
            return Err(DispatchError::UnableToPayCost);
        };

        player.set_dice_after_paying_cast(phc!(self, active_player_id), &d);
        Ok(())
    }

    pub fn check_switch_is_fast_action(&self, player_id: PlayerId, src_char_idx: u8) -> bool {
        let player = self.players.get(player_id);
        let mut res = false;
        let sc = &player.status_collection;
        if sc.responds_to(RespondsTo::SwitchIsFastAction) {
            sc.consume_statuses_immutable(
                CharIdxSelector::One(src_char_idx),
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
                    CharIdxSelector::One(src_char_idx),
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
                        CharIdxSelector::All,
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
                    player.clear_flags_for_end_of_turn(phc!(self, player_id));
                    for (char_idx, char_state) in player.char_states.iter_all_mut().enumerate() {
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
        let ctx_for_dmg = self.ctx_for_dmg(src_player_id, ctx.src);
        let src_player = self.players.get_mut(src_player_id);
        // When switching to a new character -> set plunging attack flag
        if let EventId::Switched = event_id {
            let CommandSource::Switch {
                from_char_idx,
                dst_char_idx: char_idx,
            } = ctx.src
            else {
                panic!("trigger_event: EventId::Switched: Failed to match for ctx.src: CommandSource::Switch.");
            };
            src_player.char_states[from_char_idx]
                .remove_flag_hashed(chc!(self, src_player_id, from_char_idx), CharFlag::PlungingAttack);
            src_player.char_states[char_idx]
                .insert_flag_hashed(chc!(self, src_player_id, char_idx), CharFlag::PlungingAttack);
        }

        let mut cmds = cmd_list![];
        if src_player.status_collection.responds_to_trigger_event(event_id) {
            let src_player_state = &view!(src_player);
            let sicb = StatusImplContextBuilder::new(src_player_state, ctx, ());
            mutate_statuses_1!(phc!(self, src_player_id), src_player, |sc| {
                sc.consume_statuses(
                    CharIdxSelector::All,
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
                        CharIdxSelector::All,
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
            if tgt_player.is_valid_char_idx(tgt_player.active_char_idx) {
                Some(CommandTarget::new(opp, tgt_player.active_char_idx))
            } else {
                None
            }
        };
        CommandContext::new(src_player_id, src, cmd_tgt)
    }

    fn do_switch_character(&mut self, ctx: &CommandContext, char_idx: u8) -> ExecResult {
        let p = self.players.get_mut(ctx.src_player_id);
        let prev_char_idx = p.active_char_idx;
        // Switching into self or invalid character does nothing
        if !p.switch_character_hashed(phc!(self, ctx.src_player_id), char_idx) {
            return ExecResult::Success;
        }
        ExecResult::AdditionalCmds(cmd_list![Self::trigger_switch_cmd(
            ctx.src_player_id,
            prev_char_idx,
            char_idx
        ),])
    }

    fn switch_relative(&mut self, ctx: &CommandContext, switch_type: RelativeCharIdx) -> ExecResult {
        let Some(char_idx) = self.players[ctx.src_player_id].relative_switch_char_idx(switch_type) else {
            return ExecResult::Success;
        };
        self.do_switch_character(ctx, char_idx)
    }

    fn apply_element_to_self(&mut self, ctx: &CommandContext, elem: Element) -> ExecResult {
        let char_idx = if let CommandSource::Skill { char_idx, .. } = ctx.src {
            char_idx
        } else {
            self.players.get(ctx.src_player_id).active_char_idx
        };

        let src_player = self.players.get_mut(ctx.src_player_id);
        let Some(src_char) = src_player.try_get_character_mut(char_idx) else {
            return ExecResult::Success;
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
        let (Some(tgt_char_idx), Some(tgt_player_id)) = (ctx.get_dmg_tgt_char_idx(), ctx.get_dmg_tgt_player_id())
        else {
            panic!("deal_dmg: No dst_char for ctx");
        };

        let log = &mut self.log;
        if tgt_player_id != ctx.src_player_id.opposite() {
            panic!("deal_dmg: Invalid tgt_player_id");
        }

        let (src_player_id, tgt_player_id) = (ctx.src_player_id, ctx.src_player_id.opposite());
        let (src_player, tgt_player) = self.players.get_two_mut(src_player_id);
        if !tgt_player.is_valid_char_idx(tgt_char_idx) {
            return ExecResult::Success;
        }

        let mut defeated = CharIdxSet::default();
        let mut addl_cmds: SmallVec<[(CommandContext, Command); 8]> = cmd_list![];
        let mut i = 0usize;
        let mut targets: SmallVec<[_; 4]> = smallvec![(tgt_char_idx, dmg)];
        while i < targets.len() {
            let (tgt_char_idx, mut dmg) = targets[i];
            let is_piercing = dmg.dmg_type == DealDMGType::Piercing;
            if !tgt_player.char_states.is_valid_char_idx(tgt_char_idx) {
                i += 1;
                continue;
            }

            let (tgt_applied, log_tgt, dmg_info) = {
                let tgt_char = &tgt_player.char_states[tgt_char_idx];
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

            let tgt_char_idx = {
                let mut tgt_char_idx = tgt_char_idx;
                let PlayerState {
                    char_states: tgt_char_states,
                    active_char_idx: tgt_active_char_idx,
                    ..
                } = &tgt_player;
                apply_statuses!(
                    (src_player, src_player_id, RespondsTo::OutgoingDMGTarget, dmg_info),
                    |sc_src, sicb| augment_outgoing_dmg_target_for_statuses(
                        sc_src,
                        sicb,
                        tgt_char_states,
                        *tgt_active_char_idx,
                        &dmg,
                        &mut tgt_char_idx
                    )
                );
                tgt_char_idx
            };

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
                        let (dmg_bonus, piercing, rxn_cmd) = if matches!(reaction, Some((Reaction::Bloom, _)))
                            && src_player
                                .status_collection
                                .has_team_status(StatusId::GoldenChalicesBounty)
                        {
                            (1, 0, Some(Command::Summon(SummonId::BountifulCore)))
                        } else {
                            rxn.reaction_effects(te)
                        };
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
                let tgt_char = &mut tgt_player.char_states[tgt_char_idx];
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
                let tgt_char = &mut tgt_player.char_states[tgt_char_idx];
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
                let new_char_states: &mut CharStates = &mut tgt_player.char_states;

                let pd = dmg.piercing_dmg_to_standby;
                for (j, new_char_state) in new_char_states.enumerate_valid_mut() {
                    if tgt_char_idx == j {
                        continue;
                    }

                    let pdmg = DealDMG::new(DealDMGType::Physical, pd, 0);
                    {
                        let defeated = reduce_hp(new_char_state, j, pdmg.dmg);
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
                Some(Command::InternalDealSwirlDMG(e, ..)) => {
                    for (j, _) in tgt_player
                        .char_states
                        .enumerate_valid()
                        .filter(|&(j, _)| j != tgt_char_idx)
                    {
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

    fn deal_dmg_relative(&mut self, ctx: &CommandContext, dmg: DealDMG, relative: RelativeCharIdx) -> ExecResult {
        let (Some(tgt_player_id), Some(tgt_char_idx)) = (ctx.get_dmg_tgt_player_id(), ctx.get_dmg_tgt_char_idx())
        else {
            panic!("deal_dmg_relative(relative={relative:?}): cmd has no target.");
        };
        let tgt_char_idx = self.players[tgt_player_id]
            .relative_switch_char_idx(relative)
            .unwrap_or(tgt_char_idx);
        self.deal_dmg(
            &ctx.with_tgt(Some(CommandTarget::new(tgt_player_id, tgt_char_idx))),
            dmg,
        )
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
            let char_state = &mut player.char_states[char_idx];
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
            let mut ctx = *ctx;
            ctx.src_player_id = player_id;
            addl_cmds.push((
                ctx,
                Command::InternalApplyCharacterStatusWithStateToActive(status_id, eff_state),
            ));
        }
    }

    fn take_dmg(&mut self, ctx: &CommandContext, dmg: DealDMG) -> ExecResult {
        // TODO refactor TakeDMG targeting
        let take_dmg_player_id = ctx.src_player_id;
        let char_idx = ctx.src.char_idx().unwrap_or_else(|| {
            let player = self.get_player(take_dmg_player_id);
            player.active_char_idx
        });
        let ctx = CommandContext::new(
            take_dmg_player_id.opposite(),
            ctx.src,
            Some(CommandTarget::new(take_dmg_player_id, char_idx)),
        );
        self.deal_dmg(&ctx, dmg)
    }

    fn take_dmg_for_affected_by(&mut self, ctx: &CommandContext, status_id: StatusId, dmg: DealDMG) -> ExecResult {
        let player = self.players.get_mut(ctx.src_player_id);
        let sc = &player.status_collection;
        let mut cmds = smallvec![];
        for (char_idx, _c) in player.char_states.enumerate_valid() {
            if !sc.has_character_status(char_idx, status_id) {
                continue;
            }

            let ctx = CommandContext::new(ctx.src_player_id, CommandSource::Character { char_idx }, None);
            cmds.push((ctx, Command::TakeDMG(dmg)));
        }
        ExecResult::AdditionalCmds(cmds)
    }

    fn add_energy_without_maximum(&mut self, ctx: &CommandContext, energy: u8) -> ExecResult {
        let check = |c: &CharState| !c.is_invalid() && c.get_energy() < c.char_id.get_char_card().max_energy;
        let char_idx = {
            let p = self.players.get(ctx.src_player_id);
            let active_char = p.get_active_character();
            if check(active_char) {
                Some(p.active_char_idx)
            } else {
                p.char_states.enumerate_valid().find(|(_, c)| check(c)).map(|(i, _)| i)
            }
        };

        if let Some(char_idx) = char_idx {
            self.add_energy(ctx, energy, char_idx.into())
        } else {
            ExecResult::Success
        }
    }

    fn add_energy(&mut self, ctx: &CommandContext, energy: u8, char_idx: CmdCharIdx) -> ExecResult {
        let char_idx = self.resolve_cmd_char_idx(ctx, char_idx);
        let p = self.players.get_mut(ctx.src_player_id);
        if let Some(active_char) = p.try_get_character_mut(char_idx) {
            active_char.add_energy_hashed(chc!(self, ctx.src_player_id, char_idx), energy);
        }
        ExecResult::Success
    }

    fn add_energy_to_non_active_characters(&mut self, ctx: &CommandContext, energy: u8) -> ExecResult {
        let player = self.players.get_mut(ctx.src_player_id);
        let active_char_idx = player.active_char_idx;
        for (i, char_state) in player.char_states.enumerate_valid_mut() {
            let char_idx = i;
            if char_idx == active_char_idx {
                continue;
            }
            char_state.add_energy_hashed(chc!(self, ctx.src_player_id, char_idx), energy);
        }
        ExecResult::Success
    }

    fn set_energy_for_active_character(&mut self, ctx: &CommandContext, energy: u8) -> ExecResult {
        let char_idx = self.resolve_cmd_char_idx(ctx, CmdCharIdx::Active);
        let p = self.players.get_mut(ctx.src_player_id);
        if let Some(active_char) = p.try_get_character_mut(char_idx) {
            active_char.set_energy_hashed(chc!(self, ctx.src_player_id, char_idx), energy);
        }
        ExecResult::Success
    }

    fn shift_energy_to_active_character(&mut self, ctx: &CommandContext) -> ExecResult {
        let char_idx = self.resolve_cmd_char_idx(ctx, CmdCharIdx::Active);
        let player = self.players.get_mut(ctx.src_player_id);
        let mut total = 0;
        for (i, char_state) in player.char_states.enumerate_valid_mut() {
            let i = i;
            if i == char_idx || char_state.get_energy() == 0 {
                continue;
            }
            char_state.set_energy_hashed(chc!(self, ctx.src_player_id, i), char_state.get_energy() - 1);
            total += 1;
            if total >= 2 {
                break;
            }
        }
        let char_state = &mut player.char_states[char_idx];
        let new_energy = min(
            char_state.get_energy() + total,
            char_state.char_id.get_char_card().max_energy,
        );
        char_state.set_energy_hashed(chc!(self, ctx.src_player_id, char_idx), new_energy);
        ExecResult::Success
    }

    fn increase_status_usages(&mut self, ctx: &CommandContext, key: StatusKey, usages: u8) -> ExecResult {
        if self.get_player(ctx.src_player_id).status_collection.get(key).is_none() {
            return ExecResult::Success;
        }

        mutate_statuses!(self, ctx.src_player_id, |sc| {
            let eff_state = sc.get_mut(key).expect("Status key must be present.");
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
        });
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

    fn heal(&mut self, ctx: &CommandContext, hp: u8, char_idx: CmdCharIdx) -> ExecResult {
        let char_idx = self.resolve_cmd_char_idx(ctx, char_idx);
        let Some(active_char) = self.players.get_mut(ctx.src_player_id).try_get_character_mut(char_idx) else {
            return ExecResult::Success;
        };
        active_char.heal_hashed(chc!(self, ctx.src_player_id, char_idx), hp);
        if self.log.enabled {
            self.log
                .log(Event::Heal(ctx.src_player_id, (char_idx, active_char.char_id), hp));
        }
        ExecResult::Success
    }

    fn heal_taken_most_dmg(&mut self, ctx: &CommandContext, hp: u8) -> ExecResult {
        let char_states = &self.players.get(ctx.src_player_id).char_states;
        let Some((char_idx, _)) = char_states.get_taken_most_dmg() else {
            return ExecResult::Success;
        };
        self.heal(
            &ctx.with_src(CommandSource::Character { char_idx }),
            hp,
            char_idx.into(),
        )
    }

    fn heal_all(&mut self, ctx: &CommandContext, hp: u8) -> ExecResult {
        let p = self.players.get_mut(ctx.src_player_id);
        for (char_idx, character) in p.char_states.enumerate_valid_mut() {
            character.heal_hashed(chc!(self, ctx.src_player_id, char_idx), hp);
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

    pub(crate) fn add_cards_to_hand(&mut self, player_id: PlayerId, cards: &CappedLengthList8<CardId>) -> ExecResult {
        let player = self.players.get_mut(player_id);
        for card_id in cards.to_vec_copy() {
            player.add_card_to_hand(phc!(self, player_id), card_id);
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
        if let Some(player_id) = self.phase.active_player() {
            let player = &mut self.players[player_id];
            let c = chc!(self, player_id, player.active_char_idx);
            let char = player.get_active_character_mut();
            char.remove_flag_hashed(c, CharFlag::PlungingAttack);
        }

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

    fn apply_character_status_to_target(&mut self, ctx: &CommandContext, status_id: StatusId) -> ExecResult {
        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Character {
            panic!("apply_status_to_target: wrong StatusAttachMode");
        }

        if !status.applies_to_opposing {
            panic!("apply_status_to_target: applies_to_opposing is false");
        }

        let Some(tgt_player_id) = ctx.get_dmg_tgt_player_id() else {
            panic!("apply_status_to_target: no target");
        };
        let tgt_player = self.players.get_mut(tgt_player_id);
        let tgt_char_idx = ctx.get_dmg_tgt_char_idx().unwrap_or(tgt_player.active_char_idx);
        if !tgt_player.is_valid_char_idx(tgt_char_idx) {
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

    fn apply_team_status_to_target_player(&mut self, ctx: &CommandContext, status_id: StatusId) -> ExecResult {
        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Team {
            panic!("apply_status_to_target_player: wrong StatusAttachMode");
        }
        if !status.applies_to_opposing {
            panic!("apply_status_to_target_player: applies_to_opposing is false");
        }
        let Some(tgt_player_id) = ctx.get_dmg_tgt_player_id() else {
            panic!("apply_status_to_target_player: no target player");
        };

        self.log.log(Event::ApplyTeamStatus(tgt_player_id, status_id));
        self.apply_or_refresh_status(tgt_player_id, StatusKey::Team(status_id), status);
        ExecResult::Success
    }

    fn apply_character_status_to_all_opponent_characters(
        &mut self,
        ctx: &CommandContext,
        status_id: StatusId,
    ) -> ExecResult {
        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Character {
            panic!("apply_character_status_to_all_opponent_characters: wrong StatusAttachMode");
        }
        if !status.applies_to_opposing {
            panic!("apply_character_status_to_all_opponent_characters: applies_to_opposing is false");
        }
        let Some(tgt_player_id) = ctx.get_dmg_tgt_player_id() else {
            panic!("apply_character_status_to_all_opponent_characters: no target");
        };

        let tgt_char_states = &self.players[tgt_player_id].char_states;
        let to_apply: Vector<(u8, CharId)> = tgt_char_states.enumerate_valid().map(|(i, c)| (i, c.char_id)).collect();
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

    fn apply_character_status(
        &mut self,
        ctx: &CommandContext,
        status_id: StatusId,
        char_idx: CmdCharIdx,
    ) -> ExecResult {
        let char_idx = self.resolve_cmd_char_idx(ctx, char_idx);
        let player = self.players.get_mut(ctx.src_player_id);
        if !player.is_valid_char_idx(char_idx) {
            return ExecResult::Success;
        }

        let status = status_id.get_status();
        if status.attach_mode != StatusAttachMode::Character {
            panic!("apply_character_status: wrong StatusAttachMode");
        }
        self.apply_or_refresh_status(ctx.src_player_id, StatusKey::Character(char_idx, status_id), status);

        let log = &mut self.log;
        if log.enabled {
            let player = self.players.get_mut(ctx.src_player_id);
            log.log(Event::ApplyCharStatus(
                ctx.src_player_id,
                (
                    char_idx,
                    player.try_get_character(char_idx).expect("try_get_character").char_id,
                ),
                status_id,
            ));
        }
        ExecResult::Success
    }

    fn apply_equipment(
        &mut self,
        ctx: &CommandContext,
        slot: EquipSlot,
        status_id: StatusId,
        char_idx: CmdCharIdx,
    ) -> ExecResult {
        let char_idx = self.resolve_cmd_char_idx(ctx, char_idx);
        let player = self.players.get_mut(ctx.src_player_id);
        if !player.is_valid_char_idx(char_idx) {
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
                (
                    char_idx,
                    player.try_get_character(char_idx).expect("try_get_character").char_id,
                ),
                slot,
                Some(status_id),
            ));
        }
        ExecResult::Success
    }

    fn apply_talent_to_character(
        &mut self,
        ctx: &CommandContext,
        status_id: Option<StatusId>,
        char_idx: CmdCharIdx,
    ) -> ExecResult {
        let char_idx = self.resolve_cmd_char_idx(ctx, char_idx);
        let player = self.players.get_mut(ctx.src_player_id);
        if !player.is_valid_char_idx(char_idx) {
            return ExecResult::Success;
        }
        let char_state = &mut player.char_states[char_idx];
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
                (
                    char_idx,
                    player.try_get_character(char_idx).expect("try_get_character").char_id,
                ),
                EquipSlot::Talent,
                status_id,
            ));
        }
        ExecResult::Success
    }

    fn apply_character_status_with_state_to_active(
        &mut self,
        player_id: PlayerId,
        status_id: StatusId,
        eff_state: AppliedEffectState,
    ) -> ExecResult {
        let active_char_idx = self.get_player(player_id).active_char_idx;
        mutate_statuses!(self, player_id, |sc| {
            sc.set_status(StatusKey::Character(active_char_idx, status_id), eff_state)
        });
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

    fn force_switch_for_target(&mut self, ctx: &CommandContext, force_switch_type: RelativeCharIdx) -> ExecResult {
        let Some(tgt_player_id) = ctx.get_dmg_tgt_player_id() else {
            panic!("force_switch_for_target: no target");
        };
        let tgt_player = self.players.get_mut(tgt_player_id);
        let tgt_char_idx = ctx.get_dmg_tgt_char_idx().unwrap_or(tgt_player.active_char_idx);

        if tgt_player.active_char_idx != tgt_char_idx {
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
            if player.is_valid_char_idx(player.active_char_idx) {
                continue;
            }

            // Found a winner due to all characters being dead
            if player.char_states.iter_all().all(CharState::is_invalid) {
                return ExecResult::Return(DispatchResult::Winner(player_id.opposite()));
            }

            // Ask the player to switch character after death
            let prev_addl_cmds = match prev_res {
                ExecResult::AdditionalCmds(ac) => Some(ac),
                _ => None,
            };

            player.insert_flag(phc!(self, player_id), PlayerFlag::DiedThisRound);
            return ExecResult::Suspend(SuspendedState::post_death_switch(player_id), prev_addl_cmds);
        }

        prev_res
    }

    fn stellar_restoration_from_skill(&mut self, ctx: &CommandContext) -> ExecResult {
        let player_id = ctx.src_player_id;
        let active_player = self.players.get_mut(player_id);
        let char_idx = active_player.active_char_idx;
        let mut h = ZobristHasher::new();
        let res = if active_player.try_remove_card_from_hand((&mut h, player_id), CardId::LightningStiletto) {
            ExecResult::AdditionalCmds(cmd_list![(
                *ctx,
                Command::ApplyCharacterStatus(StatusId::ElectroInfusion, char_idx.into())
            )])
        } else {
            if !matches!(ctx.src, CommandSource::Card { .. }) {
                active_player.add_card_to_hand((&mut h, player_id), CardId::LightningStiletto);
                if active_player.is_tactical() {
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
        let char_idx = ctx.src.char_idx().unwrap_or(player.active_char_idx);
        let chc = chc!(self, player_id, char_idx);
        {
            let char = &mut player.char_states[char_idx];
            let flags = char.flags | char.skill_flags(skill_id);
            char.set_flags_hashed(chc, flags);
        }
        let cmds = get_cast_skill_cmds(player, ctx, skill_id);
        ExecResult::AdditionalCmds(cmds)
    }

    #[inline]
    fn resolve_cmd_char_idx(&self, ctx: &CommandContext, char_idx: CmdCharIdx) -> u8 {
        match char_idx {
            CmdCharIdx::Active => self.players.get(ctx.src_player_id).active_char_idx,
            CmdCharIdx::CardSelected => match ctx.src {
                CommandSource::Card { target: Some(CardSelection::OwnCharacter(char_idx)), .. } => char_idx,
                _ => panic!("resolve_cmd_char_idx: CmdCharIdx::CardSelected must be used on card effects with own character selections. ctx.src={:?}", ctx.src),
            }
            CmdCharIdx::Index(char_idx) => char_idx,
        }
    }

    fn exec(&mut self, ctx: &CommandContext, cmd: Command) -> ExecResult {
        let res: ExecResult = match cmd {
            Command::Nop => ExecResult::Success,
            Command::CastSkill(skill_id) => self.cast_skill_from_cmd(ctx, skill_id),
            Command::TriggerEvent(event_id) => self.trigger_event(ctx, event_id),
            Command::TriggerXEvent(xevt) => self.trigger_xevent(ctx, xevt),
            Command::SwitchCharacter(char_idx) => self.do_switch_character(ctx, char_idx),
            Command::ApplyElementToSelf(elem) => self.apply_element_to_self(ctx, elem),
            Command::DealDMG(dmg) => self.deal_dmg(ctx, dmg),
            Command::TakeDMG(dmg) => self.take_dmg(ctx, dmg),
            Command::DealDMGRelative(dmg, relative) => self.deal_dmg_relative(ctx, dmg, relative),
            Command::TakeDMGForAffectedBy(status_id, dmg) => self.take_dmg_for_affected_by(ctx, status_id, dmg),
            Command::InternalDealSwirlDMG(..) => panic!("Cannot execute InternalDealSwirlDMG command."),
            Command::Heal(hp, char_idx) => self.heal(ctx, hp, char_idx),
            Command::HealTakenMostDMG(hp) => self.heal_taken_most_dmg(ctx, hp),
            Command::HealAll(hp) => self.heal_all(ctx, hp),
            Command::AddEnergy(energy, char_idx) => self.add_energy(ctx, energy, char_idx),
            Command::AddEnergyWithoutMaximum(energy) => self.add_energy_without_maximum(ctx, energy),
            Command::AddEnergyToNonActiveCharacters(energy) => self.add_energy_to_non_active_characters(ctx, energy),
            Command::SetEnergyForActiveCharacter(energy) => self.set_energy_for_active_character(ctx, energy),
            Command::ShiftEnergyToActiveCharacter => self.shift_energy_to_active_character(ctx),
            Command::IncreaseStatusUsages(key, usages) => self.increase_status_usages(ctx, key, usages),
            Command::DeleteStatus(key) => self.delete_status(ctx, key),
            Command::DeleteStatusForTarget(key) => self.delete_status_for_target(ctx, key),
            Command::RerollDice => self.reroll_dice(ctx),
            Command::AddDice(dice) => self.add_dice(ctx, &dice),
            Command::SubtractDice(dice) => self.subtract_dice(ctx, &dice),
            Command::AddCardsToHand(cards) => self.add_cards_to_hand(ctx.src_player_id, &cards),
            Command::DrawCards(count, card_type) => self.draw_cards(ctx, count, card_type),
            Command::ApplyCharacterStatus(status_id, char_idx) => self.apply_character_status(ctx, status_id, char_idx),
            Command::ApplyEquipment(slot, status_id, char_idx) => self.apply_equipment(ctx, slot, status_id, char_idx),
            Command::ApplyTalent(status_id, char_idx) => self.apply_talent_to_character(ctx, status_id, char_idx),
            Command::InternalApplyCharacterStatusWithStateToActive(status_id, eff_state) => {
                self.apply_character_status_with_state_to_active(ctx.src_player_id, status_id, eff_state)
            }
            Command::AddSupport(slot, support_id) => self.add_support(ctx, slot, support_id),
            Command::ApplyStatusToTeam(status_id) => self.apply_status_to_team(ctx, status_id),
            Command::ApplyCharacterStatusToTarget(status_id) => self.apply_character_status_to_target(ctx, status_id),
            Command::ApplyTeamStatusToTargetPlayer(status_id) => {
                self.apply_team_status_to_target_player(ctx, status_id)
            }
            Command::ApplyCharacterStatusToAllOpponentCharacters(status_id) => {
                self.apply_character_status_to_all_opponent_characters(ctx, status_id)
            }
            Command::Summon(summon_id) => self.summon(ctx, summon_id),
            Command::SummonRandom(spec) => self.summon_random(ctx, spec),
            Command::SwitchPrev => self.switch_relative(ctx, RelativeCharIdx::Previous),
            Command::SwitchNext => self.switch_relative(ctx, RelativeCharIdx::Next),
            Command::ForceSwitchForTarget(force_switch_type) => self.force_switch_for_target(ctx, force_switch_type),
            Command::HandOverPlayer => self.hand_over_player(),
            Command::EndOfTurn => self.end_of_turn(),
            Command::InternalStellarRestorationFromSkill => self.stellar_restoration_from_skill(ctx),
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
            let (ctx, cmd) = queue.pop_front().expect("queue is empty");
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
        self.players
            .0
            .check_for_charged_attack(phc!(self, PlayerId::PlayerFirst));
        self.players
            .1
            .check_for_charged_attack(phc!(self, PlayerId::PlayerSecond));

        if let Some(active_player) = self.phase.active_player() {
            DispatchResult::PlayerInput(active_player)
        } else {
            DispatchResult::NoInput
        }
    }
}
