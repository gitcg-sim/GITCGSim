//! # Genius Invokation TCG Simulator
//!
//! This crate implements functionalities of the Genius Invokation TCG:
//! - Game state representation and evolution
//! - Move generation and validation
//! - Minimax and MCTS search algorithms
//!
//! ## Using the `GameState` type
//! ```
//! use gitcg_sim::prelude::*;
//! use gitcg_sim::vector; // SmallVec used throughout the library
//! use gitcg_sim::list8; // List with up to 8 elements
//!
//! // Create a new GameState
//! let mut game_state: GameState = GameStateBuilder::default()
//!     .with_characters(
//!        vector![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Bennett],
//!        vector![CharId::Fischl, CharId::RhodeiaOfLoch, CharId::FatuiPyroAgent]
//!     )
//!     .skip_to_roll_phase()
//!     .build();
//!
//! // Waiting for nondeterministic input, so no player is to move
//! assert_eq!(None, game_state.to_move_player());
//!
//! // Required to initialize.
//! game_state.advance(Input::NoAction).unwrap();
//!
//! // Add cards to both players hands
//! game_state
//!     .advance(Input::NondetResult(NondetResult::ProvideCards(
//!         list8![CardId::LeaveItToMe, CardId::Starsigns],
//!         list8![CardId::Strategize, CardId::Paimon],
//!     )))
//!     .unwrap();
//!
//! // Add 8 Omni dice to both players
//! game_state
//!     .advance(Input::NondetResult(NondetResult::ProvideDice(
//!         DiceCounter::omni(8),
//!         DiceCounter::omni(8),
//!     )))
//!     .unwrap();
//!
//! println!("{:?}", game_state.available_actions());
//! ```
//!
//! ## Hashing and mutation
//!
//! The game state is hashed incrementally through [Zobrist hashing]. If the game state is updated manually
//! outside of `advance`, `game_state.rehash()` must be called to recopmute the has.
//!
//! ```
//! use gitcg_sim::prelude::*;
//! use gitcg_sim::{vector, list8};
//!
//! // Create a new GameState
//! let mut game_state: GameState = GameStateBuilder::default()
//!     .with_characters(
//!        vector![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Bennett],
//!        vector![CharId::Fischl, CharId::RhodeiaOfLoch, CharId::FatuiPyroAgent]
//!     )
//!     .skip_to_roll_phase()
//!     .build();
//!
//! // Get the Zobrist hash
//! game_state.zobrist_hash();
//! // Perform an external update
//! game_state.players[PlayerId::PlayerFirst].hand.push(CardId::QuickKnit);
//! // Recalculate the hash
//! game_state.rehash();
//! ```
//! ## Serialization and deserialization
//!
//! `serde` is supported for the game state representation.
//!
//! ## Handling non-determinism
//!
//! The `GameStateWrapper` type handles non-determinism automatically using a player decks and an existing RNG.
//!
//! ```
//! use gitcg_sim::prelude::*;
//! use gitcg_sim::{vector, list8};
//! use gitcg_sim::rand::{rngs::SmallRng, SeedableRng}; // Re-exports of rand crate
//!
//! let deck1 = Decklist::new(vector![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Bennett], vec![/* CardId::... */].into());
//! let deck2 = Decklist::new(vector![CharId::Fischl, CharId::RhodeiaOfLoch, CharId::FatuiPyroAgent], vec![].into());
//! let rng = SmallRng::seed_from_u64(100).into();
//!
//! // Nondet provider based on deck and RNG
//! let nd = NondetProvider::new(StandardNondetHandlerState::new(&deck1, &deck2, rng));
//! // This nondet provider that does nothing
//! // let nd_state = NondetProvider::new(EmptyNondetState());
//!
//! let game_state: GameState = GameStateBuilder::default()
//!     .with_characters(deck1.characters, deck2.characters)
//!     .skip_to_roll_phase()
//!     .build();
//! let game_state_wrapper = GameStateWrapper::new(game_state, nd);
//! ```
//!
//! The `new_standard_game` function bypass most intermediate steps for constructing a `GameStateWrapper`.
//!
//! ```
//! use gitcg_sim::prelude::*;
//! use gitcg_sim::{vector, list8};
//! use gitcg_sim::rand::{rngs::SmallRng, SeedableRng}; // Re-exports of rand crate
//!
//! let deck1 = Decklist::new(vector![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Bennett], vec![/* CardId::... */].into());
//! let deck2 = Decklist::new(vector![CharId::Fischl, CharId::RhodeiaOfLoch, CharId::FatuiPyroAgent], vec![].into());
//! let rng = SmallRng::seed_from_u64(100).into();
//! let game_state_wrapper = new_standard_game(&deck1, &deck2, rng);
//! ```

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

/// Datatypes for the Genious Invokation TCG domain
pub mod tcg_model;

pub mod types;

pub(crate) mod zobrist_hash;

/// Builder for `GameState`s
pub mod builder;

/// Re-exports the `smallvec` crate
pub mod smallvec {
    pub use smallvec::*;
}

/// Re-exports the `rand` create
pub mod rand {
    pub use rand::*;
}

/// Re-exports the `enum_map` crate
pub mod enum_map {
    pub use enum_map::*;
}

/// Re-exports the `enumset` crate
pub mod enumset {
    pub use enumset::*;
}

pub mod game_state_types {
    pub use crate::types::dice_counter::DiceCounter;
    pub use crate::types::game_state::{
        CharState, GameState, Phase, PlayerFlag, PlayerId, PlayerState, StatusCollection,
    };
    pub use crate::types::logging::EventLog;
    pub use crate::types::status_impl::RespondsTo;
}

pub mod prelude {
    pub use crate::builder::*;
    pub use crate::deck::Decklist;
    pub use crate::dispatcher_ops::types::DispatchError;
    pub use crate::game_state_types::*;
    pub use crate::game_tree_search::{new_standard_game, GameStateWrapper};
    pub use crate::ids::*;
    pub use crate::types::input::{Input, NondetResult, PlayerAction};
    pub use crate::types::nondet::*;
}

pub mod playout;

#[cfg(feature = "training")]
pub mod training;

#[cfg(test)]
mod tests;
