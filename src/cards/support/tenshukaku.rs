use super::*;

use crate::decl_support_impl_type;

pub const NAME: &str = "Tenshukaku";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::aligned(2),
    effects: list8![],
    card_type: CardType::Support(SupportType::Location),
    card_impl: Some(&SupportImpl(SupportId::Tenshukaku)),
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Support, 2, None);

decl_support_impl_type!(Tenshukaku, I);
impl StatusImpl for Tenshukaku {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::TriggerEvent]
    }

    fn responds_to_triggers(&self) -> EnumSet<EventId> {
        enum_set![EventId::StartOfActionPhase]
    }

    fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
        let EventId::StartOfActionPhase = e.event_id else {
            return None;
        };
        if e.c.src_player_state.dice.distinct_count() < 5 {
            return None;
        }
        e.add_cmd(Command::AddDice(DiceCounter::omni(1)));
        Some(AppliedEffectResult::ConsumeUsage)
    }
}
