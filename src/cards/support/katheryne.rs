use super::*;

use crate::decl_support_impl_type;

pub const NAME: &str = "Katheryne";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::unaligned(2),
    effects: list8![],
    card_type: CardType::Support(SupportType::Companion),
    card_impl: Some(&SupportImpl(SupportId::Katheryne)),
};

pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Support);

decl_support_impl_type!(Katheryne, I);
impl StatusImpl for Katheryne {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::SwitchIsFastAction]
    }

    fn switch_is_fast_action(&self, eff_state: &AppliedEffectState, res: &mut bool) -> Option<AppliedEffectResult> {
        if eff_state.can_use_once_per_round() {
            *res = true;
            Some(AppliedEffectResult::ConsumeOncePerRound)
        } else {
            None
        }
    }
}
