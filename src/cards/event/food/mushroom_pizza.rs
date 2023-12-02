use super::*;

pub const C: Card = Card {
    name: "Mushroom Pizza",
    cost: Cost::ONE,
    effects: list8![
        Command::Heal(1, CmdCharIdx::CardSelected),
        Command::ApplyCharacterStatus(StatusId::MushroomPizza, CmdCharIdx::Active)
    ],
    card_type: CardType::Food,
    card_impl: Some(&FoodCardImpl()),
};

pub const S: Status = Status::new_usages("Mushroom Pizza", StatusAttachMode::Character, 2, None);

// TODO tests
decl_status_impl_type!(MushroomPizza, I);
impl StatusImpl for MushroomPizza {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::TriggerEvent]
    }

    fn responds_to_triggers(&self) -> EnumSet<EventId> {
        enum_set![EventId::EndPhase]
    }

    fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
        e.out_cmds
            .push((*e.ctx_for_dmg, Command::Heal(1, e.attached_cmd_char_idx())));
        Some(AppliedEffectResult::ConsumeUsage)
    }
}
