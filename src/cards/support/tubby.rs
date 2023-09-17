use crate::decl_support_impl_type;

use super::*;

pub const NAME: &str = "Tubby";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::aligned(2),
    effects: list8![],
    card_type: CardType::Support(SupportType::Companion),
    card_impl: Some(&SupportImpl(SupportId::Tubby)),
};

pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Support);

decl_support_impl_type!(Tubby, I);
impl StatusImpl for Tubby {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::UpdateCost]
    }

    fn update_cost(&self, e: &StatusImplContext, cost: &mut Cost, cost_type: CostType) -> Option<AppliedEffectResult> {
        let CostType::Card(card_id) = cost_type else {
            return None;
        };

        if !e.eff_state.can_use_once_per_round() {
            return None;
        }

        if card_id.get_card().card_type != CardType::Support(SupportType::Location) {
            return None;
        }

        if cost.try_reduce_by(2) {
            Some(AppliedEffectResult::ConsumeOncePerRound)
        } else {
            None
        }
    }
}
