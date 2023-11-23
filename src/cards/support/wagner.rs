use super::*;

pub const NAME: &str = "Wagner";

pub const C: Card = Card {
    name: NAME,
    cost: Cost::aligned(2),
    effects: list8![],
    card_type: CardType::Support(SupportType::Companion),
    card_impl: Some(&SupportImpl(SupportId::Wagner)),
};

pub const S: Status =
    Status::new_indef(NAME, StatusAttachMode::Support).with_counter(CounterSpec::new("Forging Billets", 2));

pub const I: CardCostReductionSupport = CardCostReductionSupport {
    card_type: CardTypeFilter::Weapon,
    end_phase_counter_gain: 1,
};
