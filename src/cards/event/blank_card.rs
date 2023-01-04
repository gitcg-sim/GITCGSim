use super::*;

pub const C: Card = Card {
    name: "Blank Card",
    cost: Cost::ZERO,
    effects: list8![],
    card_type: CardType::Event,
    card_impl: Some(&I),
};

pub struct BlankCard();
impl CardImpl for BlankCard {
    fn can_be_played(&self, _: &CardImplContext) -> CanBePlayedResult {
        CanBePlayedResult::CannotBePlayed
    }
}

pub const I: BlankCard = BlankCard();
