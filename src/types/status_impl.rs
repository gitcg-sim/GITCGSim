use enumset::{EnumSet, EnumSetType};

use crate::ids::SkillId;
use crate::status_impl_trait_decl;

use super::{
    card_defs::Cost,
    char_state::CharStates,
    command::*,
    deal_dmg::DealDMG,
    dice_counter::DiceDistribution,
    enums::{Element, Reaction},
    game_state::{AppliedEffectResult, AppliedEffectState},
    StatusSpecModifier,
};

#[derive(Debug, PartialOrd, Ord, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[enumset(repr = "u16")]
pub enum RespondsTo {
    UpdateCost,
    UpdateStatusSpec,
    IncomingDMG,
    OutgoingDMG,
    OutgoingDMGTarget,
    LateOutgoingDMG,
    OutgoingReactionDMG,
    GainsEnergy,
    DiceDistribution,
    MultiplyOutgoingDMG,
    TriggerEvent,
    TriggerXEvent,
    PreparingSkill,
    CannotPerformActions,
    /// Switch from (not to) the active character is Fast Action
    SwitchIsFastAction,
}

status_impl_trait_decl!(
    /// Programmatic implementation for a status, which comes from:
    ///  - Character passive
    ///  - Team status
    ///  - Character status
    ///  - Summon
    ///  - Equipment
    ///
    /// Invariants for methods that return `Option<AppliedEffectResult>`:
    /// - If the return value is `None`:
    ///   - The activation conditions for that method has not been met
    ///   - The `&mut` parameters must not be changed
    /// - If the return value is `Some(..)`:
    ///   - The activation conditions for that method has been met
    #[allow(unused_variables)]
    pub trait StatusImpl {
        fn responds_to(&self) -> EnumSet<RespondsTo>;

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            EnumSet::empty()
        }

        fn responds_to_events(&self) -> XEventMask {
            Default::default()
        }

        fn update_status_spec(&self, modifiers: &mut StatusSpecModifier) -> bool {
            false
        }

        fn update_cost(
            &self,
            e: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            None
        }

        fn gains_energy(&self, e: &StatusImplContext, ctx_for_skill: &CommandContext, gains_energy: &mut bool) -> bool {
            false
        }

        fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            None
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            None
        }

        fn outgoing_dmg_target(
            &self,
            e: &StatusImplContext<DMGInfo>,
            tgt_chars: &CharStates,
            tgt_active_char_idx: u8,
            dmg: &DealDMG,
            tgt_char_idx: &mut u8,
        ) -> Option<AppliedEffectResult> {
            None
        }

        /// Like `outgoing_dmg`, but is called after all other `outgoing_dmg`.
        /// Used for updating post-infusion DMG.
        fn late_outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            None
        }

        fn outgoing_reaction_dmg(
            &self,
            e: &StatusImplContext<DMGInfo>,
            reaction: (Reaction, Option<Element>),
            dmg: &mut DealDMG,
        ) -> Option<AppliedEffectResult> {
            None
        }

        fn multiply_dmg(&self, e: &StatusImplContext<DMGInfo>, mult: &mut u8) -> Option<AppliedEffectResult> {
            None
        }

        fn dice_distribution(&self, e: &StatusImplContext, dist: &mut DiceDistribution) -> bool {
            false
        }

        fn switch_is_fast_action(&self, eff_state: &AppliedEffectState, res: &mut bool) -> Option<AppliedEffectResult> {
            None
        }

        fn preparing_skill(&self, eff_state: &AppliedEffectState) -> Option<SkillId> {
            None
        }

        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
            None
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            None
        }
    }
);

pub struct EmptyStatusImpl();

impl StatusImpl for EmptyStatusImpl {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        EnumSet::empty()
    }
}

#[macro_export]
macro_rules! decl_status_impl_type {
    ($name: ident $(, $impl_name: ident)?) => {
        pub struct $name();
        impl $name {
            // Ensure status id is valid
            #[allow(dead_code)]
            pub const STATUS_ID: $crate::cards::ids::StatusId = $crate::cards::ids::StatusId::$name;
        }

        $(pub const $impl_name : $name = $name (); )?
    };
}

#[macro_export]
macro_rules! decl_summon_impl_type {
    ($name: ident $(, $impl_name: ident)?) => {
        pub struct $name();
        impl $name {
            // Ensure summon id is valid
            #[allow(dead_code)]
            pub const SUMMON_ID: $crate::cards::ids::SummonId = $crate::cards::ids::SummonId::$name;
        }

        $(pub const $impl_name : $name = $name (); )?
    };
}
