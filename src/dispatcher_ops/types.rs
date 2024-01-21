use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NondetRequest {
    DrawCards(ByPlayer<u8>),
    DrawCardsOfType(PlayerId, u8, Option<CardType>),
    RollDice(ByPlayer<DiceDistribution>),
    SummonRandom(SummonRandomSpec),
}

/// Indicates game state advancement succeeds.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DispatchResult {
    Winner(PlayerId),
    NoInput,
    NondetRequest(NondetRequest),
    PlayerInput(PlayerId),
}

/// Indicates game state advancement fails due to input validation errors.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum DispatchError {
    #[cfg_attr(feature = "std", error("invalid input: {0}"))]
    InvalidInput(&'static str),
    #[cfg_attr(feature = "std", error("nondet result not allowed"))]
    NondetResultNotAllowed,
    #[cfg_attr(feature = "std", error("nondet result required"))]
    NondetResultRequired,
    #[cfg_attr(feature = "std", error("invalid nondet result"))]
    NondetResultInvalid,
    #[cfg_attr(feature = "std", error("cannot switch into character"))]
    CannotSwitchInto,
    #[cfg_attr(feature = "std", error("invalid player ID"))]
    InvalidPlayer,
    #[cfg_attr(feature = "std", error("invalid skill ID"))]
    InvalidSkillId,
    #[cfg_attr(feature = "std", error("invalid switch character index"))]
    InvalidSwitchIndex,
    #[cfg_attr(feature = "std", error("cannot cast skills here"))]
    CannotCastSkills,
    #[cfg_attr(feature = "std", error("unable to pay cost for this action"))]
    UnableToPayCost,
    #[cfg_attr(feature = "std", error("card is not on hand"))]
    CardNotOnHand,
    #[cfg_attr(feature = "std", error("cannot play card"))]
    UnableToPlayCard,
    #[cfg_attr(feature = "std", error("invalid selecction target"))]
    InvalidSelection,
}
