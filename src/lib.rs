pub(crate) mod card_impls;
pub mod cards;

/// Contains collection data types used to implement the Genius Invokation TCG.
pub(crate) mod data_structures;
pub mod deck;
pub mod dice_counter;
pub mod dispatcher;
pub mod dispatcher_ops;
pub mod game_tree_search;

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

pub mod smallvec {
    pub use smallvec::*;
}

pub mod rand {
    pub use rand::*;
}

pub mod enum_map {
    pub use enum_map::*;
}

pub mod enumset {
    pub use enumset::*;
}

#[cfg(test)]
mod tests;
