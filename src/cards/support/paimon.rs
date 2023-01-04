use super::*;

pub const NAME: &str = "Paimon";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::aligned(3),
    effects: list8![],
    card_type: CardType::Support(SupportType::Companion),
    card_impl: Some(&SupportImpl(SupportId::Paimon)),
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Support, 2, None);

pub const I: EndPhaseCommands = EndPhaseCommands(list8![Command::AddDice(DiceCounter::omni(2))]);
