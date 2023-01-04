use super::*;

use crate::decl_support_impl_type;

pub const NAME: &str = "Dawn Winery";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::aligned(2),
    effects: list8![],
    card_type: CardType::Support(SupportType::Location),
    card_impl: Some(&SupportImpl(SupportId::DawnWinery)),
};

pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Support);

decl_support_impl_type!(DawnWinery, I);
impl StatusImpl for DawnWinery {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::UpdateCost]
    }

    fn update_cost(&self, e: &StatusImplContext, cost: &mut Cost, cost_type: CostType) -> Option<AppliedEffectResult> {
        if !(cost_type == CostType::Switching && e.eff_state.can_use_once_per_round()) {
            return None;
        }
        if cost.try_reduce_by(1) {
            Some(AppliedEffectResult::ConsumeOncePerRound)
        } else {
            None
        }
    }
}
