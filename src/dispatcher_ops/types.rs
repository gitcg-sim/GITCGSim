use crate::types::{
    card_defs::CardType, command::SummonRandomSpec, dice_counter::distribution::DiceDistribution, game_state::PlayerId,
};

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
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DispatchError {
    InvalidInput(String),
    NondetResultNotAllowed,
    NondetResultRequired,
    NondetResultInvalid,
    CannotSwitchInto,
    InvalidPlayer,
    InvalidSkillId,
    InvalidSwitchIndex,
    CannotCastSkills,
    UnableToPayCost,
    CardNotOnHand,
    UnableToPlayCard,
    InvalidSelection,
}
