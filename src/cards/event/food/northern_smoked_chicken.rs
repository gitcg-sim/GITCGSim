use super::*;

const NAME: &str = "Northern Smoked Chicken";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ZERO,
    effects: list8![Command::ApplyStatusToActiveCharacter(StatusId::NorthernSmokedChicken)],
    card_type: CardType::Food,
    card_impl: Some(&FoodCardImpl()),
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Character, 1, None);

decl_status_impl_type!(NorthernSmokedChicken, I);
impl StatusImpl for NorthernSmokedChicken {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::UpdateCost]
    }

    fn update_cost(&self, _: &StatusImplContext, cost: &mut Cost, cost_type: CostType) -> Option<AppliedEffectResult> {
        let Some(SkillType::NormalAttack) = cost_type.get_skill().map(|s| s.skill_type) else {
            return None
        };

        if cost.try_reduce_unaligned_cost(1) {
            Some(AppliedEffectResult::ConsumeUsage)
        } else {
            None
        }
    }
}
