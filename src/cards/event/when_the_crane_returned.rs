use super::*;

const NAME: &str = "When the Crane Returned";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ONE,
    effects: list8![Command::DrawCards(2, None)],
    card_type: CardType::Event,
    card_impl: None,
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Team, 1, None);

decl_event_handler_trait_impl!(OwnCharacterSkillEvent(WhenTheCraneReturned), I);
impl OwnCharacterSkillEvent for WhenTheCraneReturned {
    fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
        e.add_cmd(Command::SwitchNext);
        Some(AppliedEffectResult::ConsumeUsage)
    }
}
