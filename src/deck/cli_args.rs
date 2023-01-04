use std::{fs::File, path::PathBuf, str::FromStr};
use structopt::StructOpt;

use crate::{
    game_tree_search::*,
    mcts::{MCTSConfig, MCTS},
    minimax::{MinimaxConfig, MinimaxSearch},
    rule_based::RuleBasedSearch,
    types::{
        game_state::PlayerId,
        nondet::{NondetState, StandardNondetHandlerState},
    },
};

use super::{read_decklist_from_file, Decklist};

#[derive(Debug, Copy, Clone)]
pub enum SearchAlgorithm {
    Minimax,
    MCTS,
    RuleBasedSearch,
}

impl FromStr for SearchAlgorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimax" => Ok(Self::Minimax),
            "mcts" => Ok(Self::MCTS),
            "rule-based" => Ok(Self::RuleBasedSearch),
            _ => Err(""),
        }
    }
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Genius Invokation TCG simulator")]
pub struct DeckOpt {
    #[structopt(
        parse(from_os_str),
        short = "a",
        long = "--player1-deck",
        help = "Path to the deck for Player 1."
    )]
    pub player1_deck: PathBuf,

    #[structopt(
        parse(from_os_str),
        short = "b",
        long = "--player2-deck",
        help = "Path to the deck for Player 2."
    )]
    pub player2_deck: PathBuf,

    #[structopt(
        short = "R",
        long = "--random-decks",
        help = "Randomize both players characters and decks"
    )]
    pub random_decks: bool,

    #[structopt(short = "s", long = "--steps", help = "Benchmark steps")]
    pub steps: Option<u32>,

    #[structopt(
        short = "A",
        long = "--algorithm",
        help = "minimax|mcts|rule-based, Minimax/Monte-Carlo Tree Search: algorithm used for the game tree search."
    )]
    pub algorithm: Option<SearchAlgorithm>,

    #[structopt(short = "d", long = "--depth", help = "Minimax: search depth")]
    pub search_depth: Option<u8>,

    #[structopt(short = "D", long = "--mcts-debug", help = "MCTS: print debug info")]
    pub mcts_debug: bool,

    #[structopt(
        short = "T",
        long = "--mcts-time-limit-ms",
        help = "MCTS: set time limit per move in milliseconds"
    )]
    pub mcts_time_limit_ms: Option<u32>,

    #[structopt(short = "C", long = "--mcts-c", help = "MCTS: search constant")]
    pub mcts_c: Option<f32>,

    #[structopt(
        short = "I",
        long = "--mcts-iters",
        help = "MCTS: number of playouts per playout step"
    )]
    pub mcts_random_playout_iters: Option<u32>,

    #[structopt(short = "M", long = "--mcts-max-steps", help = "MCTS: max steps per playout")]
    pub mcts_random_playout_max_steps: Option<u32>,

    #[structopt(short = "S", long = "--seed", help = "Random seed for the game states")]
    pub seed: Option<u64>,
}

pub enum GenericSearch<S: NondetState = StandardNondetHandlerState> {
    Minimax(MinimaxSearch<GameStateWrapper<S>>),
    MCTS(MCTS<GameStateWrapper<S>>),
    RuleBasedSearch(RuleBasedSearch),
}

impl<S: NondetState> GenericSearch<S> {
    pub fn search(
        &mut self,
        position: &GameStateWrapper<S>,
        maximize_player: PlayerId,
    ) -> SearchResult<GameStateWrapper<S>> {
        match self {
            Self::Minimax(s) => s.search(position, maximize_player),
            Self::MCTS(s) => s.search(position, maximize_player),
            Self::RuleBasedSearch(s) => s.search(position, maximize_player),
        }
    }
}

impl DeckOpt {
    pub fn get_player1_deck(&self) -> Result<Decklist, std::io::Error> {
        let f = File::open(&self.player1_deck)?;
        read_decklist_from_file(f)
    }

    pub fn get_player2_deck(&self) -> Result<Decklist, std::io::Error> {
        let f = File::open(&self.player2_deck)?;
        read_decklist_from_file(f)
    }

    pub fn make_search<S: NondetState>(&self, parallel: bool) -> GenericSearch<S> {
        match self.algorithm.unwrap_or(SearchAlgorithm::Minimax) {
            SearchAlgorithm::Minimax => {
                let config = MinimaxConfig::new(self.search_depth.unwrap_or(6), parallel, None);
                GenericSearch::Minimax(MinimaxSearch::new(config))
            }
            SearchAlgorithm::MCTS => {
                let config = MCTSConfig::new(
                    self.mcts_time_limit_ms.unwrap_or(2000),
                    self.mcts_c.unwrap_or(2.0),
                    crate::minimax::transposition_table::DEFAULT_SIZE,
                    parallel,
                    self.mcts_random_playout_iters.unwrap_or(500),
                    self.mcts_random_playout_max_steps.unwrap_or(200),
                    self.mcts_debug,
                );
                GenericSearch::MCTS(MCTS::new(config))
            }
            SearchAlgorithm::RuleBasedSearch => {
                GenericSearch::RuleBasedSearch(RuleBasedSearch::new(Default::default()))
            }
        }
    }
}
