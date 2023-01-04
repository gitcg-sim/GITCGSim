use crate::cards::ids::*;
use crate::data_structures::CommandList;
use crate::status_impls::primitives::all::*;
use crate::{decl_support_impl_type, list8};

use crate::types::dice_counter::distribution::DiceDistribution;
use crate::types::{card_defs::*, card_impl::*, command::*, dice_counter::*, enums::*, game_state::*, status_impl::*};

use enumset::{enum_set, EnumSet};

pub mod paimon;

pub mod dawn_winery;

pub mod katheryne;

pub mod iron_tongue_tian;

pub mod liu_su;

pub mod favonius_cathedral;

pub mod wangshu_inn;

pub mod liyue_harbor_wharf;

pub mod timaeus;

pub mod wagner;

pub mod tubby;

pub mod jade_chamber;

pub mod knights_of_favonius_library;

pub mod liben;

pub mod sangonomiya_shrine;

pub mod tenshukaku;

pub struct SupportImpl(pub SupportId);

impl CardImpl for SupportImpl {
    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        if cic.get_next_available_suport_slot().is_some() {
            CanBePlayedResult::CanBePlayed
        } else {
            // TODO support replacing an existing support when there's not enough room
            CanBePlayedResult::CannotBePlayed
        }
    }

    fn selection(&self) -> Option<CardSelectionSpec> {
        // TODO support replacing
        None
    }

    fn get_effects(
        &self,
        cic: &CardImplContext,
        ctx: &CommandContext,
        commands: &mut CommandList<(CommandContext, Command)>,
    ) {
        let Some(slot) = cic.get_next_available_suport_slot() else {
            // TODO support replacing
            return
        };
        for eff in cic.card.effects.to_vec() {
            commands.push((*ctx, eff))
        }
        // TODO card's own commands
        commands.push((*ctx, Command::AddSupport(slot, self.0)));
    }
}

pub enum CardTypeFilter {
    Weapon,
    Artifact,
    Talent,
}

impl CardTypeFilter {
    #[inline]
    pub fn matches(&self, card_type: CardType) -> bool {
        match (self, card_type) {
            (Self::Weapon, CardType::Weapon(..)) => true,
            (Self::Artifact, CardType::Artifact) => true,
            (Self::Talent, CardType::Talent(..)) => true,
            (Self::Weapon, _) => false,
            (Self::Artifact, _) => false,
            (Self::Talent, _) => false,
        }
    }
}

pub struct CardCostReductionSupport {
    pub card_type: CardTypeFilter,
    pub end_phase_counter_gain: u8,
}

impl StatusImpl for CardCostReductionSupport {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::UpdateCost | RespondsTo::TriggerEvent]
    }

    fn responds_to_triggers(&self) -> EnumSet<EventId> {
        enum_set![EventId::EndPhase]
    }

    fn update_cost(&self, e: &StatusImplContext, cost: &mut Cost, cost_type: CostType) -> Option<AppliedEffectResult> {
        let CostType::Card(card_id) = cost_type else {
            return None
        };

        if !e.eff_state.can_use_once_per_round() {
            return None;
        }

        if !self.card_type.matches(card_id.get_card().card_type) {
            return None;
        }

        let counter = e.eff_state.get_counter();
        let total = cost.total_dice();
        if counter >= total && cost.try_reduce_by(counter) {
            Some(AppliedEffectResult::SetCounterAndConsumeOncePerRound(counter - total))
        } else {
            None
        }
    }

    fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
        let EventId::EndPhase = e.event_id else {
            return None
        };

        let counter = e.c.eff_state.get_counter();
        Some(AppliedEffectResult::SetCounter(counter + self.end_phase_counter_gain))
    }
}

#[macro_export]
macro_rules! decl_support_impl_type {
    ($name: ident $(, $impl_name: ident)?) => {
        pub struct $name();
        impl $name {
            // Ensure support id is valid
            #[allow(dead_code)]
            pub const SUPPORT_ID: $crate::cards::ids::SupportId = $crate::cards::ids::SupportId::$name;
        }

        $(pub const $impl_name : $name = $name (); )?
    };
}
