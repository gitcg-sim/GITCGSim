#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![doc = include_str!("../README.md")]

#[macro_export]
#[doc(hidden)]
macro_rules! impl_display_from_debug {
    (@single $Type: ident) => {
        impl $crate::std_subset::fmt::Display for $Type {
            fn fmt(&self, f: &mut $crate::std_subset::fmt::Formatter<'_>) -> $crate::std_subset::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
    ($($Type: ident)+) => {
        $(impl_display_from_debug!(@single $Type);)+
    };
}

pub(crate) mod std_subset {
    pub use core::{cmp, hash, iter, marker, mem, ops};
    pub use smallvec::alloc::{boxed::Box, collections, fmt, slice, string::String, sync, vec, vec::Vec};
}

pub(crate) mod card_impls;

/// Module containing the definitions of cards.
pub mod cards;

/// Module containing collection datatypes used by this crate.
pub mod data_structures;
pub mod deck;
pub mod dice_counter;
pub(crate) mod dispatcher;
pub(crate) mod dispatcher_ops;

/// Module containing `enums` that identify Genius Invokation TCG entities.
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

/// Elemental reaction
pub(crate) mod reaction;

/// Pseudorandom number generation
pub mod rng;

/// Implementation for rule-based TCG player
pub mod rule_based;

pub(crate) mod status_impls;

/// Module containing the enums and DMG representation for Genius Invokation TCG
pub mod tcg_model;

pub(crate) mod builder_macros;

pub(crate) mod types;

pub(crate) mod zobrist_hash;

pub(crate) mod game_state_wrapper;

/// Builder for `GameState`s
pub mod builder;

/// Re-exports the `smallvec` crate
pub use smallvec;

/// Re-exports the `rand` create
pub use rand;

/// Re-exports the `enum_map` crate
pub use enum_map;

/// Re-exports the `thiserror` crate
#[cfg(feature = "std")]
pub use thiserror;

/// Re-exports the `enumset` crate
pub use enumset;

pub(crate) mod iter_helpers;

// TODO move this
pub mod minimax_eval;

pub mod prelude {
    pub use crate::builder::*;
    pub use crate::deck::Decklist;
    pub use crate::dispatcher_ops::{DispatchError, DispatchResult, NondetRequest};
    pub use crate::game_state_wrapper::{new_standard_game, GameStateWrapper};
    pub use crate::ids::*;
    pub use crate::types::by_player::ByPlayer;
    pub use crate::types::dice_counter::{DiceCounter, DiceDeterminization, DiceDistribution, ElementPriority};
    pub use crate::types::game_state::{
        AppliedEffectState, CardSelection, CharState, GameState, PendingCommands, Phase, PlayerFlag, PlayerId,
        PlayerState, StatusCollection, StatusEntry, StatusKey, SuspendedState,
    };
    pub use crate::types::input::{Input, NondetResult, PlayerAction};
    pub use crate::types::logging::EventLog;
    pub use crate::types::nondet::{
        EmptyNondetState, NondetProvider, NondetState, StandardNondetHandlerFlags, StandardNondetHandlerState,
    };
    pub use crate::types::status_impl::RespondsTo;
    pub use crate::zobrist_hash::{HashValue, ZobristHashable};

    // Modules
    pub use crate::types::card_defs;
    pub use crate::types::logging;
    pub use crate::types::tcg_model;
}

#[cfg(test)]
mod tests;
