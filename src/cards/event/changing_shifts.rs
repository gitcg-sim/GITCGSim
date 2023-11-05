use super::*;

pub const C: Card = Card {
    name: "Changing Shifts",
    cost: Cost::ZERO,
    effects: list8![Command::ApplyStatusToTeam(StatusId::ChangingShifts)],
    card_type: CardType::Event,
    card_impl: None,
};

pub const S: Status = Status::new_usages("Changing Shifts", StatusAttachMode::Team, 1, None);

decl_status_impl_type!(ChangingShifts, I);
impl StatusImpl for ChangingShifts {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::UpdateCost]
    }

    fn update_cost(&self, _: &StatusImplContext, cost: &mut Cost, cost_type: CostType) -> Option<AppliedEffectResult> {
        let CostType::Switching = cost_type else { return None };
        cost.try_reduce_by(1).then_some(AppliedEffectResult::ConsumeUsage)
    }
}
