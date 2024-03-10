use crate::data_structures::List8;
use crate::status_impls::prelude::*;

pub mod dmg {
    use super::*;
    #[macro_export]
    #[doc(hidden)]
    macro_rules! increase_outgoing_dmg_impl {
        ($Type: ident, |$e: ident, $dmg: ident| $check: expr) => {
            impl StatusImpl for $Type {
                fn responds_to(&self) -> EnumSet<RespondsTo> {
                    enum_set![RespondsTo::OutgoingDMG]
                }

                #[allow(unused_variables)]
                fn outgoing_dmg(
                    &self,
                    $e: &StatusImplContext<DMGInfo>,
                    $dmg: &mut DealDMG,
                ) -> Option<AppliedEffectResult> {
                    let check = { $check };
                    if !check {
                        return None;
                    }
                    $dmg.dmg += self.dmg_increase;
                    Some(self.result)
                }
            }
        };
    }

    pub struct IncreaseOutgoingDMG {
        pub dmg_increase: u8,
        pub result: AppliedEffectResult,
    }

    impl IncreaseOutgoingDMG {
        pub const fn new(dmg_increase: u8, result: AppliedEffectResult) -> Self {
            Self { dmg_increase, result }
        }
    }

    increase_outgoing_dmg_impl!(IncreaseOutgoingDMG, |_e, _dmg| true);

    pub struct IncreaseChargedAttackDMG {
        pub dmg_increase: u8,
        pub result: AppliedEffectResult,
    }

    impl IncreaseChargedAttackDMG {
        pub const fn new(dmg_increase: u8, result: AppliedEffectResult) -> Self {
            Self { dmg_increase, result }
        }
    }

    increase_outgoing_dmg_impl!(IncreaseChargedAttackDMG, |e, _dmg| e.is_charged_attack());
}

pub mod incoming_dmg {
    use super::*;

    pub struct ReduceDMGAbove {
        pub dmg_reduction: u8,
        pub receive_at_least_dmg: Option<u8>,
        pub result: AppliedEffectResult,
    }

    impl ReduceDMGAbove {
        pub const fn new(dmg_reduction: u8, receive_at_least_dmg: Option<u8>, result: AppliedEffectResult) -> Self {
            Self {
                dmg_reduction,
                receive_at_least_dmg,
                result,
            }
        }
    }

    impl StatusImpl for ReduceDMGAbove {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG]
        }

        fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if let Some(min_dmg) = self.receive_at_least_dmg {
                if dmg.dmg < min_dmg {
                    return None;
                }
            }

            dmg.reduce(self.dmg_reduction).then_some(self.result)
        }
    }
}

pub mod end_phase {
    use super::*;

    /// Implementation for a status that deals DMG
    /// with usage counts at the End Phase.
    ///
    /// End Phase: Deal [DMG], Usages: [count]
    pub struct EndPhaseDealDMG(pub DealDMG);

    impl StatusImpl for EndPhaseDealDMG {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            e.cmd_deal_dmg(self.0.dmg_type, self.0.dmg, self.0.piercing_dmg_to_standby);
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }

    pub enum TakeDMGCharacter {
        Attached,
        Active,
    }

    /// Implementation for a status that takes DMG to the attached/active character.
    ///
    /// End Phase: Deal [DMG] to the [active character/the charater to which this is attached].
    /// Usages: [count]
    pub struct EndPhaseTakeDMG {
        pub mode: TakeDMGCharacter,
        pub take_dmg: DealDMG,
    }

    impl EndPhaseTakeDMG {
        pub const fn new(mode: TakeDMGCharacter, take_dmg: DealDMG) -> Self {
            Self { mode, take_dmg }
        }
    }

    impl StatusImpl for EndPhaseTakeDMG {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
            let EventId::EndPhase = e.event_id else {
                return None;
            };
            let char_idx = match self.mode {
                TakeDMGCharacter::Attached => e.c.status_key.char_idx().unwrap_or_else(|| {
                    panic!(
                        "EndPhaseTakeDMG: Attached: Not a character status: {:?}",
                        e.c.status_key
                    )
                }),
                TakeDMGCharacter::Active => e.active_char_idx(),
            };
            e.out_cmds.push((
                e.ctx_for_dmg
                    .without_target()
                    .with_src(CommandSource::Character { char_idx }),
                Command::TakeDMG(self.take_dmg),
            ));
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }

    /// Implementation for a status that performs commands
    /// with usage counts at the End Phase.
    ///
    /// End Phase: [commands], Usages: [count]
    pub struct EndPhaseCommands(pub List8<Command>);

    impl StatusImpl for EndPhaseCommands {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            for &c in self.0.iter() {
                e.add_cmd(c)
            }
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod xevent {
    use super::*;
    use crate::types::char_state::*;

    #[inline]
    fn skill_types_into_masks(skill_types: EnumSet<SkillType>) -> XEventMask {
        (if skill_types.contains(SkillType::NormalAttack) {
            xevent_mask::SKILL_NA
        } else {
            xevent_mask::NONE
        }) | (if skill_types.contains(SkillType::ElementalSkill) {
            xevent_mask::SKILL_SKILL
        } else {
            xevent_mask::NONE
        }) | (if skill_types.contains(SkillType::ElementalBurst) {
            xevent_mask::SKILL_BURST
        } else {
            xevent_mask::NONE
        })
    }

    macro_rules! event_handler_trait {
        ($Trait: ident, $Wrapper: ident, $Arg: ident, $mask: expr, |$e: ident| $arg_expr: expr) => {
            pub trait $Trait {
                event_handler_trait!(@trait_consts $Arg);
                fn invoke(e: &mut TriggerEventContext<XEvent>, arg: $Arg) -> Option<AppliedEffectResult>;
            }

            pub struct $Wrapper<I: $Trait>(pub I);

            impl<I: $Trait> StatusImpl for $Wrapper<I> {
                #[inline(always)]
                fn responds_to(&self) -> enumset::EnumSet<RespondsTo> {
                    enum_set![RespondsTo::TriggerXEvent]
                }

                #[inline(always)]
                fn responds_to_events(&self) -> XEventMask {
                    event_handler_trait!(@skill_type_masks $Arg, I, $mask)
                }

                #[inline(always)]
                fn trigger_xevent(&self, $e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
                    let arg: $Arg = ($arg_expr)?;
                    event_handler_trait!(@skill_type_check $Arg, I, arg);
                    I::invoke($e, arg)
                }
            }
        };
        (@skill_type_masks XEventSkill, $I: ident, $mask: expr) => {
            $mask & skill_types_into_masks(I::SKILL_TYPES)
        };
        (@skill_type_check XEventSkill, $I: ident, $arg: ident) => {
            if !$I::SKILL_TYPES.contains($arg.skill_type()) {
                return None;
            }
        };
        (@trait_consts XEventSkill) => {
            /// Set of skill types that triggers this event.
            const SKILL_TYPES: EnumSet<SkillType> = enum_set![SkillType::NormalAttack | SkillType::ElementalSkill | SkillType::ElementalBurst];
        };
        (@skill_type_masks XEventDMG, $I: ident, $mask: expr) => {
            if $I::REACTION { $mask & xevent_mask::DMG_REACTION } else { $mask }
        };
        (@skill_type_check XEventDMG, $I: ident, $arg: ident) => {
            if $I::REACTION && $arg.reaction.is_none() {
                return None;
            }
        };
        (@trait_consts XEventDMG) => {
            /// True to require an Elemental Reaction trigger the event. Otherwise Elemental Reactions are not checked.
            const REACTION: bool = false;
        };
    }

    event_handler_trait!(
        OwnCharacterSkillEvent,
        OwnCharacterSkillEventI,
        XEventSkill,
        xevent_mask::SKILL_FROM_SELF,
        |e| e.event_skill_ensuring_own_player()
    );
    event_handler_trait!(
        OpponentCharacterSkillEvent,
        OpponentCharacterSkillEventI,
        XEventSkill,
        xevent_mask::SKILL_FROM_OPP,
        |e| e.event_skill_ensuring_opponent_player()
    );
    event_handler_trait!(
        AttachedCharacterSkillEvent,
        AttachedCharacterSkillEventI,
        XEventSkill,
        xevent_mask::SKILL_FROM_SELF,
        |e| e.event_skill_ensuring_attached_character()
    );
    event_handler_trait!(
        OwnCharacterIncomingDMGEvent,
        OwnCharacterIncomingDMGEventI,
        XEventDMG,
        xevent_mask::DMG_INCOMING,
        |e| e.incoming_dmg_ensuring_own_player()
    );
    event_handler_trait!(
        AttachedCharacterIncomingDMGEvent,
        AttachedCharacterIncomingDMGEventI,
        XEventDMG,
        xevent_mask::DMG_INCOMING,
        |e| e.incoming_dmg_ensuring_attached_character()
    );
    event_handler_trait!(
        OwnCharacterOutgoingDMGEvent,
        OwnCharacterOutgoingDMGEventI,
        XEventDMG,
        xevent_mask::DMG_OUTGOING,
        |e| e.outgoing_dmg_ensuring_own_player()
    );
    event_handler_trait!(
        AttachedCharacterOutgoingDMGEvent,
        AttachedCharacterOutgoingDMGEventI,
        XEventDMG,
        xevent_mask::DMG_OUTGOING,
        |e| e.outgoing_dmg_ensuring_attached_character()
    );

    #[macro_export]
    #[doc(hidden)]
    macro_rules! decl_event_handler_trait_impl {
        ($Trait: ident ( $Type: ident ) , $I: ident $(,)?) => {
            pub struct $Type();
            pub const $I: decl_event_handler_trait_impl!(@wrapper $Trait<$Type>) = decl_event_handler_trait_impl!(@wrapper $Trait)($Type());
        };
        (@wrapper OwnCharacterSkillEvent $(<$T: ident>)?) => { OwnCharacterSkillEventI $(<$T>)? };
        (@wrapper OpponentCharacterSkillEvent $(<$T: ident>)?) => { OpponentCharacterSkillEventI $(<$T>)? };
        (@wrapper AttachedCharacterSkillEvent $(<$T: ident>)?) => { AttachedCharacterSkillEventI $(<$T>)? };
        (@wrapper OwnCharacterIncomingDMGEvent $(<$T: ident>)?) => { OwnCharacterIncomingDMGEventI $(<$T>)? };
        (@wrapper AttachedCharacterIncomingDMGEvent $(<$T: ident>)?) => { AttachedCharacterIncomingDMGEventI $(<$T>)? };
        (@wrapper OwnCharacterOutgoingDMGEvent $(<$T: ident>)?) => { OwnCharacterOutgoingDMGEventI $(<$T>)? };
        (@wrapper AttachedCharacterOutgoingDMGEvent $(<$T: ident>)?) => { AttachedCharacterOutgoingDMGEventI $(<$T>)? };
    }
}

pub mod prepared_skill {
    use super::*;

    pub struct PreparedSkill {
        pub skill_id: SkillId,
    }

    impl PreparedSkill {
        pub const fn new(skill_id: SkillId) -> Self {
            Self { skill_id }
        }
    }

    impl StatusImpl for PreparedSkill {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::PreparingSkill | RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::Switched]
        }

        fn preparing_skill(&self, _: &AppliedEffectState) -> Option<SkillId> {
            Some(self.skill_id)
        }

        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
            let EventId::Switched = e.event_id else { return None };
            if e.status_key.char_idx().expect("PreparedSkill: must have char_idx") != e.active_char_idx() {
                Some(AppliedEffectResult::DeleteSelf)
            } else {
                None
            }
        }
    }
}

#[allow(unused_imports)]
pub mod all {
    pub use super::dmg::*;
    pub use super::end_phase::*;
    pub use super::incoming_dmg::*;
    pub use super::prepared_skill::*;
    pub use super::xevent::*;
    pub use crate::types::status_impl::EmptyStatusImpl;
    pub use crate::{compose_status_impls, decl_event_handler_trait_impl};
}
