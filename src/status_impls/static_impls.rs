use enumset::EnumSet;

use crate::cards::all_cards_reexports::*;
use crate::cards::char_reexports;
use crate::cards::ids::*;
use crate::cards::statuses::*;
use crate::cards::summons::burning_flame;
use crate::data_structures::CommandList;
use crate::ids::enums::{
    __generated_enum_cases_StatusId, __generated_enum_cases_SummonId, __generated_enum_cases_SupportId,
};
use crate::tcg_model::*;
use crate::types::card_defs::{Cost, SkillImpl};
use crate::types::card_impl::*;
use crate::types::char_state::{AppliedEffectResult, AppliedEffectState};
use crate::types::command::*;
use crate::types::dice_counter::DiceDistribution;
use crate::types::game_state::{CardSelectionSpec, PlayerState, StatusCollection};
use crate::types::status_impl::{RespondsTo, StatusImpl};
use crate::types::StatusSpecModifier;
use char_reexports::*;

/// An instance of `StatusImpl` backed by one of 3 enum types:
///  - `StatusId`
///  - `SummonId`
///  - `SupportId`
/// This types allows
/// `StatusImpl` to be statically dispatched instead of dispatching through `&dyn StatusImpl`.
///
/// ```rust, ignore
/// let status_id = StatusId::AurousBlaze;
/// let si_static: StaticStatusImpl = status_id.into();
/// assert_eq!(StaticStatusImpl::Status(status_id), si_static);
///
/// let si_dynamic: &dyn StatusImpl = status_id.status_impl();
/// assert_eq!(si_static.responds_to(), si_dynamic.responds_to());
/// ```
#[cfg(not(feature = "no_static_status_impl"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum StaticStatusImpl {
    Status(StatusId),
    Summon(SummonId),
    Support(SupportId),
}

#[cfg(not(feature = "no_static_status_impl"))]
impl From<StatusId> for StaticStatusImpl {
    fn from(value: StatusId) -> Self {
        Self::Status(value)
    }
}

#[cfg(not(feature = "no_static_status_impl"))]
impl From<SummonId> for StaticStatusImpl {
    fn from(value: SummonId) -> Self {
        Self::Summon(value)
    }
}

#[cfg(not(feature = "no_static_status_impl"))]
impl From<SupportId> for StaticStatusImpl {
    fn from(value: SupportId) -> Self {
        Self::Support(value)
    }
}

#[cfg(feature = "no_static_status_impl")]
#[derive(Copy, Clone)]
pub struct StaticStatusImpl {
    pub status_impl: &'static (dyn StatusImpl + 'static),
}

#[cfg(feature = "no_static_status_impl")]
impl From<StatusId> for StaticStatusImpl {
    fn from(value: StatusId) -> Self {
        Self {
            status_impl: value.status_impl(),
        }
    }
}

#[cfg(feature = "no_static_status_impl")]
impl From<SummonId> for StaticStatusImpl {
    fn from(value: SummonId) -> Self {
        Self {
            status_impl: value.status_impl(),
        }
    }
}

#[cfg(feature = "no_static_status_impl")]
impl From<SupportId> for StaticStatusImpl {
    fn from(value: SupportId) -> Self {
        Self {
            status_impl: value.status_impl(),
        }
    }
}

impl CardImpl for CardId {
    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        self.card_impl().unwrap_or(&DefaultCardImpl()).can_be_played(cic)
    }

    fn selection(&self) -> Option<CardSelectionSpec> {
        self.card_impl().unwrap_or(&DefaultCardImpl()).selection()
    }

    fn effects(
        &self,
        cic: &CardImplContext,
        ctx: &CommandContext,
        commands: &mut CommandList<(CommandContext, Command)>,
    ) {
        self.card_impl()
            .unwrap_or(&DefaultCardImpl())
            .effects(cic, ctx, commands)
    }
}

impl SkillImpl for SkillId {
    fn commands(
        &self,
        src_player: &PlayerState,
        status_collection: &StatusCollection,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        match self.skill().skill_impl {
            None => {}
            Some(si) => si.commands(src_player, status_collection, ctx, cmds),
        }
    }
}

macro_rules! __generated_enum_cases {
    (StatusId, $expr: expr, & $I: ident $(, | $val: ident | $blk: block $(,)?)?) => {
        __generated_enum_cases_StatusId!( $expr, & $I, $(| $val | $blk)? )
    };
    (SummonId, $expr: expr, & $I: ident $(, | $val: ident | $blk: block $(,)?)?) => {
        __generated_enum_cases_SummonId!( $expr, & $I, $(| $val | $blk)? )
    };
    (SupportId, $expr: expr, & $I: ident $(, | $val: ident | $blk: block $(,)?)?) => {
        __generated_enum_cases_SupportId!( $expr, & $I, $(| $val | $blk)? )
    };
}

macro_rules! forwarding_trait_impl {
    ($trait: path , $type: path => $etype: ident $(,)? {
        $(
            fn $fn_name: ident (
                &self $(,)?
                $($a_name: ident : $a_type: ty ),* $(,)?
            ) -> $rtype: ty
        );*
        $(;)?
    }) => {
        impl $trait for $type {
            $(
                #[inline(always)]
                fn $fn_name ( &self , $($a_name : $a_type ),* ) -> $rtype {
                    __generated_enum_cases!($etype, *self, &I, |i| {
                        i.$fn_name( $($a_name),* )
                    })
                }
            )*
        }
    }
}

macro_rules! forwarding_trait_impl_dyn {
    ($trait: path , $type: path => $field_name: ident $(,)? {
        $(
            fn $fn_name: ident (
                &self $(,)?
                $($a_name: ident : $a_type: ty ),* $(,)?
            ) -> $rtype: ty
        );*
        $(;)?
    }) => {
        #[cfg(feature = "no_static_status_impl")]
        impl $trait for $type {
            $(
                #[inline(always)]
                fn $fn_name ( &self , $($a_name : $a_type ),* ) -> $rtype {
                    self.$field_name.$fn_name( $($a_name),* )
                }
            )*
        }
    }
}

macro_rules! static_status_impl {
    ($trait: path , $type: path $(,)? {
        $(
            fn $fn_name: ident (
                &self $(,)?
                $($a_name: ident : $a_type: ty ),* $(,)?
            ) -> $rtype: ty
        );*
        $(;)?
    }) => {
        #[cfg(not(feature = "no_static_status_impl"))]
        impl $trait for $type {
            $(
                #[inline(always)]
                fn $fn_name ( &self , $($a_name : $a_type ),* ) -> $rtype {
                    match self {
                        Self::Status(i) => i.$fn_name( $($a_name),* ),
                        Self::Summon(i) => i.$fn_name( $($a_name),* ),
                        Self::Support(i) => i.$fn_name( $($a_name),* ),
                    }
                }
            )*
        }
    }
}

macro_rules! status_impl_macros {
    (
        $($macro_name: ident ($a1: path, $a2: path $(=> $a3: tt)?, *));+
        $(;)?
        * => $to_repeat: tt
    ) => {
        $(
            $macro_name!($a1, $a2 $(=> $a3)?, $to_repeat);
        )+
    }
}

status_impl_macros!(
    forwarding_trait_impl(StatusImpl, SummonId => SummonId, *);
    forwarding_trait_impl(StatusImpl, SupportId => SupportId, *);
    forwarding_trait_impl(StatusImpl, StatusId => StatusId, *);
    forwarding_trait_impl_dyn(StatusImpl, StaticStatusImpl => status_impl, *);
    static_status_impl(StatusImpl, crate::status_impls::static_impls::StaticStatusImpl, *);
    * => {
        fn responds_to(&self) -> EnumSet<RespondsTo>;
        fn responds_to_triggers(&self) -> EnumSet<EventId>;
        fn responds_to_events(&self) -> XEventMask;
        fn update_status_spec(&self, modifiers: &mut StatusSpecModifier) -> bool;
        fn update_cost(&self, e: &StatusImplContext, cost: &mut Cost, cost_type: CostType) -> Option<AppliedEffectResult>;
        fn gains_energy(&self, e: &StatusImplContext, ctx_for_skill: &CommandContext, gains_energy: &mut bool) -> bool;
        fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult>;
        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult>;
        fn late_outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult>;
        fn outgoing_reaction_dmg(
            &self,
            e: &StatusImplContext<DMGInfo>,
            reaction: (Reaction, Option<Element>),
            dmg: &mut DealDMG,
        ) -> Option<AppliedEffectResult>;
        fn multiply_dmg(&self, e: &StatusImplContext<DMGInfo>, mult: &mut u8) -> Option<AppliedEffectResult>;
        fn dice_distribution(&self, e: &StatusImplContext, dist: &mut DiceDistribution) -> bool;
        fn switch_is_fast_action(&self, eff_state: &AppliedEffectState, res: &mut bool) -> Option<AppliedEffectResult>;
        fn preparing_skill(&self, eff_state: &AppliedEffectState) -> Option<SkillId>;
        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult>;
        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult>;
    }
);

#[cfg(not(feature = "no_static_status_impl"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_status_id() {
        let status_id = StatusId::AurousBlaze;
        let si_static: StaticStatusImpl = status_id.into();
        assert_eq!(StaticStatusImpl::Status(status_id), si_static);

        let si_dynamic: &dyn StatusImpl = status_id.status_impl();
        assert_eq!(si_static.responds_to(), si_dynamic.responds_to());
    }
}
