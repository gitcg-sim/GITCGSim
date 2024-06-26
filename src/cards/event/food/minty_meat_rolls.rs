use super::*;

pub const NAME: &str = "Minty Meat Rolls";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ONE,
    effects: list8![Command::ApplyCharacterStatus(
        StatusId::MintyMeatRolls,
        CmdCharIdx::Active
    )],
    card_type: CardType::Food,
    card_impl: Some(&FoodCardImpl()),
};

pub const S: Status = Status::new_duration(NAME, StatusAttachMode::Character, 1);

decl_status_impl_type!(MintyMeatRolls, I);
impl StatusImpl for MintyMeatRolls {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::UpdateCost]
    }

    fn update_cost(&self, _: &StatusImplContext, cost: &mut Cost, cost_type: CostType) -> Option<AppliedEffectResult> {
        let Some(SkillType::NormalAttack) = cost_type.skill().map(|s| s.skill_type) else {
            return None;
        };

        cost.try_reduce_unaligned_cost(1)
            .then_some(AppliedEffectResult::NoChange)
    }
}
