use super::*;

pub const C: Card = Card {
    name: "Calx's Arts",
    cost: Cost::ONE,
    card_type: CardType::Event,
    card_impl: None,
    effects: list8![Command::ShiftEnergy,],
};
