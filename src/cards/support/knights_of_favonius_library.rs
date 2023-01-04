use super::*;

pub const NAME: &str = "Knights of Favonius Library";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::ONE,
    effects: list8![],
    card_type: CardType::Support(SupportType::Location),
    card_impl: Some(&SupportImpl(SupportId::KnightsOfFavoniusLibrary)),
};

pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Support);

decl_support_impl_type!(KnightsOfFavoniusLibrary, I);
impl StatusImpl for KnightsOfFavoniusLibrary {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::DiceDistribution]
    }

    fn dice_distribution(&self, _: &StatusImplContext, dist: &mut DiceDistribution) -> bool {
        dist.rerolls += 1;
        true
    }
}
