pub(crate) mod card_impls;
pub mod cards;
pub(crate) mod data_structures;
pub mod deck;
pub mod dice_counter;
pub mod dispatcher;
pub mod dispatcher_ops;
pub mod game_tree_search;
pub mod ids;
pub mod mcts;
pub mod minimax;
pub mod reaction;
pub mod rng;
pub mod rule_based;
pub(crate) mod status_impls;
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
