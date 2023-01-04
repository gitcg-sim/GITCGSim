use super::*;

pub const NAME: &str = "Liyue Harbor Wharf";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::aligned(2),
    effects: list8![],
    card_type: CardType::Support(SupportType::Location),
    card_impl: Some(&SupportImpl(SupportId::LiyueHarborWharf)),
};

pub const S: Status = Status::new_usages(NAME, StatusAttachMode::Support, 2, None);

pub const I: EndPhaseCommands = EndPhaseCommands(list8![Command::DrawCards(2, None)]);
