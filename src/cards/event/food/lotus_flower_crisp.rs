use super::*;

pub const NAME: &str = "Lotus Flower Crisp";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ONE,
    effects: list8![Command::ApplyStatusToCharacter(
        StatusId::LotusFlowerCrisp,
        CmdCharIdx::Active
    )],
    card_type: CardType::Food,
    card_impl: Some(&FoodCardImpl()),
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Character, 1, None);

decl_status_impl_type!(LotusFlowerCrisp, I);
impl StatusImpl for LotusFlowerCrisp {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::IncomingDMG | RespondsTo::TriggerEvent]
    }

    fn responds_to_triggers(&self) -> EnumSet<EventId> {
        enum_set![EventId::EndOfTurn]
    }

    fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
        if dmg.dmg <= 3 {
            dmg.dmg = 0;
        } else {
            dmg.dmg -= 3;
        }
        Some(AppliedEffectResult::ConsumeUsage)
    }

    fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
        let EventId::EndOfTurn = e.event_id else { return None };
        Some(AppliedEffectResult::DeleteSelf)
    }
}
