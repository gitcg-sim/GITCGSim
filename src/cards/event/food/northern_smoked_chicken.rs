use super::*;

const NAME: &str = "Northern Smoked Chicken";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ZERO,
    effects: list8![Command::ApplyStatusToCharacter(
        StatusId::NorthernSmokedChicken,
        CmdCharIdx::Active
    )],
    card_type: CardType::Food,
    card_impl: Some(&FoodCardImpl()),
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Character, 1, None);

decl_status_impl_type!(NorthernSmokedChicken, I);
impl StatusImpl for NorthernSmokedChicken {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::UpdateCost]
    }

    fn update_cost(&self, ctx: &StatusImplContext, cost: &mut Cost, _: CostType) -> Option<AppliedEffectResult> {
        if !ctx.is_normal_attack() {
            return None;
        }

        cost.try_reduce_unaligned_cost(1)
            .then_some(AppliedEffectResult::ConsumeUsage)
    }
}
