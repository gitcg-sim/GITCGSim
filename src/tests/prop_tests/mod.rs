mod generator;
use generator::*;

use proptest::prelude::*;

use crate::game_tree_search::{Game, ZobristHashable};
use crate::{dispatcher_ops::types::DispatchResult, types::input::Input};

pub mod state_evolution;

pub mod zobrist_hash;
