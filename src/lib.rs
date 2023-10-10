pub(crate) mod card_impls;
pub mod cards;

/// Contains collection data types used to implement the Genius Invokation TCG.
pub mod data_structures;
pub mod deck;
pub mod dice_counter;
pub(crate) mod dispatcher;
pub(crate) mod dispatcher_ops;
pub mod game_tree_search;

pub mod transposition_table;

/// Module containing `enums` that identify Genios Invokation TCG entities.
///
/// They include:
///  - Character
///  - Card (Event, artifact, etc.)
///  - Skill
///  - Status (character/team)
///  - Summon
///  - Support
///
/// This module affects generated code related to ID lookups and card effect implementations.
/// After adding a new entry to the enum, run `code_generator.py` to update the generated code.
pub mod ids;

/// Implementation for Monte-Carlo Tree Search
pub mod mcts;

/// Implementation for minimax search
pub mod minimax;

/// Elemental reaction
pub mod reaction;

/// Pseudorandom number generation
pub mod rng;

/// Implementation for rule-based TCG player
pub mod rule_based;

pub(crate) mod status_impls;

/// Datatypes for the Genious Invokation TCG domain.
pub mod tcg_model;

pub mod types;

pub(crate) mod zobrist_hash;

pub mod builder;

/// Re-exports the `smallvec` create.
pub mod smallvec {
    pub use smallvec::*;
}

/// Re-exports the `rand` create.
pub mod rand {
    pub use rand::*;
}

/// Re-exports the `enum_map` create.
pub mod enum_map {
    pub use enum_map::*;
}

/// Re-exports the `enumset` create.
pub mod enumset {
    pub use enumset::*;
}

pub mod game_state_types {
    pub use crate::types::game_state::{
        PlayerId,
        GameState,
        Phase,
        PlayerState,
        CharState,
        StatusCollection,
        PlayerFlag,
    };
    pub use crate::types::status_impl::RespondsTo;
    pub use crate::types::dice_counter::DiceCounter;
    pub use crate::types::logging::EventLog;
}

pub mod prelude {
    pub use crate::types::input::Input;
    pub use crate::dispatcher_ops::types::DispatchError;
    pub use crate::builder::*;
    pub use crate::game_state_types::*;
}

pub mod playout;

#[cfg(feature = "training")]
pub mod training;

#[cfg(test)]
mod tests;
