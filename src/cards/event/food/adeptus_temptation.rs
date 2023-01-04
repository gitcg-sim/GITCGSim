use super::*;

const NAME: &str = "Adeptus' Temptation";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::unaligned(2),
    effects: list8![Command::Heal(2)],
    card_type: CardType::Food,
    card_impl: Some(&FoodCardImpl()),
};

pub const S: Status = Status::new_duration(NAME, StatusAttachMode::Character, 1);

decl_status_impl_type!(AdeptusTemptation, I);
impl StatusImpl for AdeptusTemptation {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::OutgoingDMG]
    }

    fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
        let Some(SkillType::ElementalBurst) = e.skill_type() else {
            return None
        };

        dmg.dmg += 3;
        Some(AppliedEffectResult::DeleteSelf)
    }
}
