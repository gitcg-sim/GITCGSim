use super::*;

use crate::decl_support_impl_type;

pub const NAME: &str = "Wangshu Inn";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::aligned(2),
    effects: list8![],
    card_type: CardType::Support(SupportType::Location),
    card_impl: Some(&SupportImpl(SupportId::WangshuInn)),
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Support, 2, None);

decl_support_impl_type!(WangshuInn, I);
impl StatusImpl for WangshuInn {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::TriggerEvent]
    }

    fn responds_to_triggers(&self) -> EnumSet<EventId> {
        enum_set![EventId::EndPhase]
    }

    fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
        let EventId::EndPhase = e.event_id else { return None };
        // TODO heal most injured character
        e.add_cmd(Command::Heal(2));
        Some(AppliedEffectResult::ConsumeUsage)
    }
}
