mod generator;
use generator::*;

use proptest::prelude::*;

use crate::cards::ids::CardId;
use crate::game_tree_search::GameStateWrapper;
use crate::game_tree_search::{Game, ZobristHashable};
use crate::types::{game_state::GameState, nondet::StandardNondetHandlerState};
use crate::{dispatcher_ops::types::DispatchResult, types::input::Input};

pub mod state_evolution;

pub mod zobrist_hash;

pub mod serialization;
