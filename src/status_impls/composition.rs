use crate::status_impls::prelude::*;

pub trait C2 {
    fn c2<F: FnOnce() -> Self>(&self, f: F) -> Self;
}

impl C2 for bool {
    #[inline(always)]
    fn c2<F: FnOnce() -> Self>(&self, f: F) -> Self {
        *self || f()
    }
}

impl<T: Copy + EnumSetType> C2 for EnumSet<T> {
    #[inline(always)]
    fn c2<F: FnOnce() -> Self>(&self, f: F) -> Self {
        *self | f()
    }
}

impl<T: Copy> C2 for Option<T> {
    #[inline(always)]
    fn c2<F: FnOnce() -> Self>(&self, f: F) -> Self {
        self.or_else(f)
    }
}

#[macro_export]
macro_rules! c2_body {
    ($this: expr, $f: ident ( $($arg: ident),* $(,)? )) => {
        $crate::status_impls::composition::C2::c2(
            & $this.0.$f($($arg),*),
            || $this.1.$f($($arg),*)
        )
    };
}

#[macro_export]
macro_rules! c2_trait_impls {
    (| $this: ident | $body: expr, {
        $(
            fn $fn_name: ident (
                &self $(,)?
                $($a_name: ident : $a_type: ty ),* $(,)?
            ) -> $rtype: ty
        );*
        $(;)?
    }) => {
        $(
            #[inline(always)]
            fn $fn_name ( &self , $($a_name : $a_type ),* ) -> $rtype {
                let val = { let $this = self; $body };
                $crate::c2_body!(val, $fn_name ( $($a_name),* ))
            }
        )*
    }
}

#[macro_export]
macro_rules! trigger_event_impl {
    ($Type: ident, [$($event_id: ident),*], | $e: ident | $blk: block) => {
        impl StatusImpl for $Type {
            fn responds_to(&self) -> EnumSet<RespondsTo> {
                enum_set![RespondsTo::TriggerEvent]
            }

            fn responds_to_triggers(&self) -> EnumSet<EventId> {
                enum_set![$(EventId::$event_id)|*]
            }

            fn trigger_event(&self, $e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
                match $e.event_id {
                    $(EventId::$event_id)|* => {}
                    _ => return None
                };
                $blk
            }
        }
    };
}

#[macro_export]
macro_rules! compose_status_impls {
    ($TypeName: ident ( $A: expr, $B: expr, $C: expr $(,)? )) => {
        pub struct ComposeStatusImplsIntermediate();
        compose_status_impls!(ComposeStatusImplsIntermediate($A, $B));
        compose_status_impls!($TypeName(ComposeStatusImplsIntermediate(), $C));
    };
    ($TypeName: ident ( $A: expr, $B: expr $(,)? )) => {
        impl $crate::types::status_impl::StatusImpl for $TypeName {
            $crate::c2_trait_impls!(|_this| ($A, $B), {
                fn responds_to(&self) -> EnumSet<RespondsTo>;
                fn responds_to_triggers(&self) -> EnumSet<EventId>;
                fn responds_to_events(&self) -> XEventMask;
                fn update_status_spec(&self, modifiers: &mut StatusSpecModifier) -> bool;
                fn update_cost(
                    &self,
                    e: &StatusImplContext,
                    cost: &mut Cost,
                    cost_type: CostType,
                ) -> Option<AppliedEffectResult>;
                fn gains_energy(
                    &self,
                    e: &StatusImplContext,
                    ctx_for_skill: &CommandContext,
                    gains_energy: &mut bool,
                ) -> bool;
                fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult>;
                fn outgoing_dmg(
                    &self,
                    e: &StatusImplContext<DMGInfo>,
                    dmg: &mut DealDMG,
                ) -> Option<AppliedEffectResult>;
                fn outgoing_dmg_target(
                    &self,
                    e: &StatusImplContext<DMGInfo>,
                    tgt_chars: &CharStates,
                    tgt_active_char_idx: u8,
                    dmg: &DealDMG,
                    tgt_char_idx: &mut u8,
                ) -> Option<AppliedEffectResult>;
                fn late_outgoing_dmg(
                    &self,
                    e: &StatusImplContext<DMGInfo>,
                    dmg: &mut DealDMG,
                ) -> Option<AppliedEffectResult>;
                fn outgoing_reaction_dmg(
                    &self,
                    e: &StatusImplContext<DMGInfo>,
                    reaction: (Reaction, Option<Element>),
                    dmg: &mut DealDMG,
                ) -> Option<AppliedEffectResult>;
                fn multiply_dmg(&self, e: &StatusImplContext<DMGInfo>, mult: &mut u8) -> Option<AppliedEffectResult>;
                fn dice_distribution(&self, e: &StatusImplContext, dist: &mut DiceDistribution) -> bool;
                fn switch_is_fast_action(
                    &self,
                    eff_state: &AppliedEffectState,
                    res: &mut bool,
                ) -> Option<AppliedEffectResult>;
                fn preparing_skill(&self, eff_state: &AppliedEffectState) -> Option<SkillId>;
                fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult>;
                fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult>;
            });
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        cards::{
            builders::deal_elem_dmg,
            ids::{GetSkill, SkillId, StatusId},
        },
        tcg_model::DealDMGType,
        types::{
            dice_counter::DiceCounter,
            game_state::{PlayerId, StatusKey},
        },
    };
    use enumset::enum_set;
    use smallvec::smallvec;

    /// This character deals +1 DMG.
    struct A();
    impl StatusImpl for A {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, _: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }
    }

    /// When the character to which this is attached triggers an Elemental Reaction: Heal 1 HP for all your characters. (Once per Round)
    struct B();
    impl StatusImpl for B {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::DMG_OUTGOING & xevent_mask::DMG_REACTION
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let (_, _) = e.get_outgoing_dmg_ensuring_attached_character()?.reaction?;
            if !e.c.eff_state.can_use_once_per_round() {
                return None;
            }
            e.add_cmd(Command::HealAll(1));
            Some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }

    /// Reduce the unaligned cost for Normal Attacks by 1.
    /// After the opposing character uses an Elemental Burst: Gain 1 Omni dice.
    struct C();
    impl StatusImpl for C {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::UpdateCost | RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_OPP & xevent_mask::SKILL_BURST
        }

        fn update_cost(
            &self,
            _: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            let SkillType::NormalAttack = cost_type.get_skill()?.skill_type else {
                return None;
            };
            cost.try_reduce_unaligned_cost(1)
                .then_some(AppliedEffectResult::NoChange)
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let XEvent::Skill(XEventSkill { skill_id, .. }) = e.event_id else {
                return None;
            };
            let SkillType::ElementalBurst = skill_id.get_skill().skill_type else {
                return None;
            };
            e.add_cmd(Command::AddDice(DiceCounter::omni(1)));
            Some(AppliedEffectResult::NoChange)
        }
    }

    struct AB();
    struct AC();
    struct BC();
    struct CB();
    #[allow(clippy::upper_case_acronyms)]
    struct ABC();
    compose_status_impls!(AB(A(), B()));
    compose_status_impls!(AC(A(), C()));
    compose_status_impls!(BC(B(), C()));
    compose_status_impls!(CB(C(), B()));
    compose_status_impls!(ABC(AB(), C()));

    const COMMAND_CONTEXT: CommandContext = CommandContext {
        src_player_id: PlayerId::PlayerFirst,
        src: CommandSource::Skill {
            char_idx: 0,
            skill_id: SkillId::FireworkFlareUp,
        },
        tgt: None,
    };

    macro_rules! get_ctx {
        ($($dmg_info: expr)?) => {
            StatusImplContext {
                src_player_state: &crate::types::game_state::PlayerStateView {
                    active_char_idx: 0,
                    char_states: &Default::default(),
                    flags: enum_set![],
                    dice: DiceCounter::omni(0),
                    affected_by: smallvec![],
                },
                status_key: StatusKey::Character(0, StatusId::NiwabiEnshou),
                eff_state: &AppliedEffectState::from(128),
                ctx: &COMMAND_CONTEXT,
                dmg: get_ctx!(@dmg_info $($dmg_info)?),
            }
        };
        (@dmg_info) => { () };
        (@dmg_info $dmg_info: expr) => { ($dmg_info) };
    }

    macro_rules! event_ctx {
        ($ctx: ident, $event: expr) => {{
            TriggerEventContext {
                c: $ctx,
                status_key: StatusKey::Character(0, StatusId::NiwabiEnshou),
                event_id: $event,
                ctx_for_dmg: &COMMAND_CONTEXT,
                out_cmds: &mut smallvec![],
            }
        }};
    }

    #[test]
    fn test_merged_responds_to() {
        assert_eq!(
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::TriggerXEvent],
            AB().responds_to()
        );
        assert_eq!(
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::UpdateCost | RespondsTo::TriggerXEvent],
            AC().responds_to()
        );
        assert_eq!(
            enum_set![RespondsTo::TriggerXEvent | RespondsTo::UpdateCost],
            BC().responds_to()
        );
        assert_eq!(
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::UpdateCost | RespondsTo::TriggerXEvent],
            ABC().responds_to()
        );
    }

    #[test]
    fn test_merged_responds_to_events() {
        assert_eq!(
            xevent_mask::DMG_OUTGOING & xevent_mask::DMG_REACTION,
            AB().responds_to_events()
        );
        assert_eq!(
            (xevent_mask::SKILL_FROM_OPP & xevent_mask::SKILL_BURST),
            AC().responds_to_events()
        );
        assert_eq!(
            (xevent_mask::DMG_OUTGOING & xevent_mask::DMG_REACTION)
                | (xevent_mask::SKILL_FROM_OPP & xevent_mask::SKILL_BURST),
            BC().responds_to_events()
        );
        assert_eq!(
            (xevent_mask::DMG_OUTGOING & xevent_mask::DMG_REACTION)
                | (xevent_mask::SKILL_FROM_OPP & xevent_mask::SKILL_BURST),
            ABC().responds_to_events()
        );
    }

    #[test]
    fn test_merged_outgoing_dmg() {
        let ctx = get_ctx!(DMGInfo::default());
        let mut dmg = deal_elem_dmg(Element::Pyro, 2, 0);
        assert_eq!(Some(AppliedEffectResult::NoChange), ABC().outgoing_dmg(&ctx, &mut dmg));
        assert_eq!(3, dmg.dmg);
    }

    #[test]
    fn test_merged_triggered_xevent() {
        let dmg_event = XEvent::DMG(XEventDMG {
            src_player_id: PlayerId::PlayerFirst,
            tgt_char_idx: 1,
            dmg_value: 2,
            dmg_type: DealDMGType::Elemental(Element::Pyro),
            dmg_info: Default::default(),
            reaction: Some((Reaction::Vaporize, None)),
            defeated: false,
        });
        let skill_event = XEvent::Skill(XEventSkill {
            src_player_id: PlayerId::PlayerSecond,
            src_char_idx: 1,
            skill_id: SkillId::RyuukinSaxifrage,
        });

        {
            let ctx = get_ctx!();
            let ctx1 = get_ctx!();
            let mut event_ctx = event_ctx!(ctx, dmg_event);
            assert_eq!(
                Some(AppliedEffectResult::ConsumeOncePerRound),
                ABC().trigger_xevent(&mut event_ctx)
            );
            let mut event_ctx1 = event_ctx!(ctx1, skill_event);
            assert_eq!(
                Some(AppliedEffectResult::NoChange),
                ABC().trigger_xevent(&mut event_ctx1)
            );
        }
        {
            let ctx = get_ctx!();
            let ctx1 = get_ctx!();
            let mut event_ctx = event_ctx!(ctx, dmg_event);
            assert_eq!(
                Some(AppliedEffectResult::ConsumeOncePerRound),
                CB().trigger_xevent(&mut event_ctx)
            );
            let mut event_ctx1 = event_ctx!(ctx1, skill_event);
            assert_eq!(
                Some(AppliedEffectResult::NoChange),
                CB().trigger_xevent(&mut event_ctx1)
            );
        }
    }
}
