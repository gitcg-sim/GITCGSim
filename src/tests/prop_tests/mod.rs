mod generator;
use generator::*;

use proptest::prelude::*;

use crate::cards::ids::*;

use crate::game_tree_search::{Game, ZobristHashable};
use crate::prelude::*;
use crate::types::game_state::PlayerId;

use crate::{dispatcher_ops::types::DispatchResult, types::input::Input};

pub(crate) const CASES: u32 = 5_000;
// pub(crate) const CASES: u32 = 50_000;
// pub(crate) const CASES: u32 = 500_000;
// pub(crate) const CASES: u32 = 5_000_000;

pub mod state_evolution;

pub mod zobrist_hash;

#[cfg(feature = "serde")]
pub mod serialization;
