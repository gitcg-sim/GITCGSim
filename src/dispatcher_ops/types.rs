use crate::types::{
    card_defs::CardType, command::SummonRandomSpec, dice_counter::distribution::DiceDistribution, game_state::PlayerId,
};

// TODO refactor to use ByPlayer
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NondetRequest {
    DrawCards(u8, u8),
    DrawCardsOfType(PlayerId, u8, Option<CardType>),
    RollDice(DiceDistribution, DiceDistribution),
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
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DispatchError {
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    #[error("nondet result not allowed")]
    NondetResultNotAllowed,
    #[error("nondet result required")]
    NondetResultRequired,
    #[error("invalid nondet result")]
    NondetResultInvalid,
    #[error("cannot switch into character")]
    CannotSwitchInto,
    #[error("invalid player ID")]
    InvalidPlayer,
    #[error("invalid skill ID")]
    InvalidSkillId,
    #[error("invalid switch character index")]
    InvalidSwitchIndex,
    #[error("cannot cast skills here")]
    CannotCastSkills,
    #[error("unable to pay cost for this action")]
    UnableToPayCost,
    #[error("card is not on hand")]
    CardNotOnHand,
    #[error("cannot play card")]
    UnableToPlayCard,
    #[error("invalid selecction target")]
    InvalidSelection,
}
