use rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
use std::{fs::File, path::PathBuf, str::FromStr};
use structopt::StructOpt;

use crate::{
    game_tree_search::*,
    linked_list,
    mcts::{MCTSConfig, MCTS},
    minimax::{
        search::{STATIC_SEARCH_MAX_ITERS, TACTICAL_SEARCH_DEPTH, TARGET_ROUND_DELTA},
        MinimaxConfig, MinimaxSearch,
    },
    rule_based::RuleBasedSearch,
    types::{
        game_state::PlayerId,
        nondet::{NondetState, StandardNondetHandlerState},
    },
};

use super::{random_decklist, read_decklist_from_file, Decklist};

#[derive(Debug, Copy, Clone)]
pub enum SearchAlgorithm {
    Minimax,
    MCTS,
    RuleBasedSearch,
    Random,
}

impl FromStr for SearchAlgorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimax" => Ok(Self::Minimax),
            "mcts" => Ok(Self::MCTS),
            "rule-based" => Ok(Self::RuleBasedSearch),
            "random" => Ok(Self::Random),
            _ => Err(""),
        }
    }
}

#[derive(Debug, StructOpt, Clone, Default)]
pub struct SearchConfig {
    #[structopt(
        short = "A",
        long = "--algorithm",
        help = "minimax|mcts|rule-based, Minimax/Monte-Carlo Tree Search: algorithm used for the game tree search."
    )]
    pub algorithm: Option<SearchAlgorithm>,

    #[structopt(short = "d", long = "--depth", help = "Minimax: search depth")]
    pub search_depth: Option<u8>,

    #[structopt(short = "Q", long = "--tactical-depth", help = "Minimax: tactical search depth")]
    pub tactical_depth: Option<u8>,

    #[structopt(long = "--target-round-delta", help = "Minimax: target round delta")]
    pub target_round_delta: Option<u8>,

    #[structopt(long = "--static-search-iters", help = "Minimax: static search iterations")]
    pub static_search_iters: Option<u8>,

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

    #[structopt(long = "--mcts-playout-bias", help = "MCTS: random playout bias")]
    pub mcts_random_playout_bias: Option<f32>,

    #[structopt(
        short = "T",
        long = "--time-limit-ms",
        help = "Set time limit per move in milliseconds"
    )]
    pub time_limit_ms: Option<u128>,

    #[structopt(short = "P", long = "--max-positions", help = "Max positions to search")]
    pub max_positions: Option<u64>,

    #[structopt(long = "--tt-size-mb", help = "Transposition table size")]
    pub tt_size_mb: Option<u32>,

    #[structopt(short = "D", long = "--debug", help = "Print debug info")]
    pub debug: bool,
}

#[derive(Debug, StructOpt, Clone)]
pub struct DeckGen {
    #[structopt(
        parse(from_os_str),
        short = "a",
        long = "--player1-deck",
        help = "Path to the deck for Player 1."
    )]
    pub player1_deck: Option<PathBuf>,

    #[structopt(
        parse(from_os_str),
        short = "b",
        long = "--player2-deck",
        help = "Path to the deck for Player 2."
    )]
    pub player2_deck: Option<PathBuf>,

    #[structopt(
        short = "R",
        long = "--random-decks",
        help = "Randomize both players characters and decks"
    )]
    pub random_decks: bool,
}

#[derive(Debug, StructOpt, Clone)]
pub struct DeckOpts {
    #[structopt(flatten)]
    pub deck: DeckGen,

    #[structopt(short = "s", long = "--steps", help = "Benchmark max steps to play out")]
    pub steps: Option<u32>,

    #[structopt(short = "S", long = "--seed", help = "Random seed for the game states")]
    pub seed: Option<u64>,

    #[structopt(long = "--tactical", help = "Tactical mode")]
    pub tactical: bool,

    // TODO split up
    #[structopt(flatten)]
    pub search: SearchConfig,
}

impl DeckGen {
    pub fn get_decks<R: Rng>(&self, rng: &mut R) -> Result<(Decklist, Decklist), std::io::Error> {
        let mut get_deck = |path: &Option<PathBuf>| -> Result<Decklist, std::io::Error> {
            match path {
                None => {
                    if self.random_decks {
                        Ok(random_decklist(rng))
                    } else {
                        panic!("Must provide a deck or set --random-deck")
                    }
                }
                Some(path) => File::open(path).and_then(read_decklist_from_file),
            }
        };
        let deck1 = get_deck(&self.player1_deck)?;
        let deck2 = get_deck(&self.player2_deck)?;
        Ok((deck1, deck2))
    }
}

impl DeckOpts {
    pub fn get_decks(&self) -> Result<(Decklist, Decklist), std::io::Error> {
        let mut r = SmallRng::seed_from_u64(self.seed.unwrap_or(100));
        self.deck.get_decks(&mut r)
    }

    pub fn get_standard_game(&self, rng: Option<SmallRng>) -> Result<GameStateWrapper, std::io::Error> {
        let (d1, d2) = self.get_decks()?;
        let mut game = new_standard_game(
            &d1,
            &d2,
            rng.unwrap_or_else(|| SmallRng::seed_from_u64(self.seed.unwrap_or(100))),
        );
        if self.tactical {
            game.convert_to_tactical_search();
        }
        Ok(game)
    }
}

pub enum GenericSearch<S: NondetState = StandardNondetHandlerState> {
    Minimax(MinimaxSearch<GameStateWrapper<S>>),
    MCTS(MCTS<GameStateWrapper<S>>),
    RuleBasedSearch(RuleBasedSearch),
    Random,
}

fn random_search<S: NondetState>(position: &GameStateWrapper<S>) -> SearchResult<GameStateWrapper<S>> {
    let mut rng = thread_rng();
    let actions = position.actions();
    let selected = actions[rng.gen_range(0..actions.len())];
    SearchResult {
        pv: linked_list![selected],
        eval: Default::default(),
        counter: Default::default(),
    }
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
            Self::Random => random_search(position),
        }
    }
}

impl SearchConfig {
    pub fn make_search<S: NondetState>(&self, parallel: bool, limits: Option<SearchLimits>) -> GenericSearch<S> {
        match self.algorithm.unwrap_or(SearchAlgorithm::Minimax) {
            SearchAlgorithm::Minimax => {
                let config = MinimaxConfig {
                    depth: self.search_depth.unwrap_or(6),
                    parallel,
                    limits,
                    tt_size_mb: self
                        .tt_size_mb
                        .unwrap_or(crate::minimax::transposition_table::DEFAULT_SIZE_MB),
                    debug: self.debug,
                    tactical_depth: self.tactical_depth.unwrap_or(TACTICAL_SEARCH_DEPTH),
                    target_round_delta: self.target_round_delta.unwrap_or(TARGET_ROUND_DELTA),
                    static_search_max_iters: self.static_search_iters.unwrap_or(STATIC_SEARCH_MAX_ITERS),
                };
                GenericSearch::Minimax(MinimaxSearch::new(config))
            }
            SearchAlgorithm::MCTS => {
                let config = MCTSConfig {
                    c: self.mcts_c.unwrap_or(3.5),
                    random_playout_bias: self.mcts_random_playout_bias,
                    tt_size_mb: self
                        .tt_size_mb
                        .unwrap_or(crate::minimax::transposition_table::DEFAULT_SIZE_MB),
                    limits,
                    parallel,
                    random_playout_iters: self.mcts_random_playout_iters.unwrap_or(500),
                    random_playout_cutoff: self.mcts_random_playout_max_steps.unwrap_or(200),
                    debug: self.debug,
                };
                GenericSearch::MCTS(MCTS::new(config))
            }
            SearchAlgorithm::RuleBasedSearch => {
                GenericSearch::RuleBasedSearch(RuleBasedSearch::new(Default::default()))
            }
            SearchAlgorithm::Random => GenericSearch::Random,
        }
    }

    pub fn get_limits(&self) -> Option<SearchLimits> {
        if self.time_limit_ms.is_none() && self.max_positions.is_none() {
            return None;
        }
        Some(SearchLimits {
            max_time_ms: self.time_limit_ms,
            max_positions: self.max_positions,
        })
    }
}

impl DeckOpts {
    pub fn make_search<S: NondetState>(&self, parallel: bool, limits: Option<SearchLimits>) -> GenericSearch<S> {
        self.search.make_search(parallel, limits)
    }

    pub fn get_limits(&self) -> Option<SearchLimits> {
        self.search.get_limits()
    }
}
