use super::*;

pub const C: Card = Card {
    name: "Leave It to Me!",
    cost: Cost::ZERO,
    effects: list8![Command::ApplyStatusToTeam(StatusId::LeaveItToMe)],
    card_type: CardType::Event,
    card_impl: None,
};

pub const S: Status = Status::new_usages("Leave It to Me!", StatusAttachMode::Team, 1, None);

decl_status_impl_type!(LeaveItToMe, I);
impl StatusImpl for LeaveItToMe {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::SwitchIsFastAction]
    }

    fn switch_is_fast_action(&self, _ctx: &AppliedEffectState, value: &mut bool) -> Option<AppliedEffectResult> {
        *value = true;
        Some(AppliedEffectResult::ConsumeUsage)
    }
}
