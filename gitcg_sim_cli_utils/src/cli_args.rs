use std::{fs::File, path::PathBuf, str::FromStr};
use structopt::StructOpt;

use gitcg_sim::{
    deck::{random_decklist, read_decklist_from_file, Decklist},
    game_tree_search::*,
    linked_list,
    rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng},
    rule_based::RuleBasedSearch,
    types::{
        game_state::PlayerId,
        nondet::{NondetState, StandardNondetHandlerState},
    },
};
use gitcg_sim_search::{
    mcts::{policy::RuleBasedPuct, CpuctConfig, MCTSConfig, MCTS},
    minimax::{
        search::{STATIC_SEARCH_MAX_ITERS, TACTICAL_SEARCH_DEPTH, TARGET_ROUND_DELTA},
        MinimaxConfig, MinimaxSearch,
    },
    training::policy::{search::PolicyNetworkBasedSearch, PolicyNetwork},
};

#[derive(Debug, Copy, Clone)]
pub enum SearchAlgorithm {
    Minimax,
    MCTS,
    RuleBased,
    PolicyBased,
    Random,
}

impl FromStr for SearchAlgorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimax" => Ok(Self::Minimax),
            "mcts" => Ok(Self::MCTS),
            "rule-based" => Ok(Self::RuleBased),
            "policy-based" => Ok(Self::PolicyBased),
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

    #[structopt(
        short = "C",
        long = "--mcts-c",
        help = "MCTS: Cpuct base value. Higher value promotes wider search, while lower value promotes deeper search."
    )]
    pub mcts_cpuct_init: Option<f32>,

    #[structopt(
        long = "--mcts-c-base",
        help = "MCTS: Cpuct growth rate scaling. Higher value reduces Cpuct growth."
    )]
    pub mcts_cpuct_base: Option<f32>,

    #[structopt(
        long = "--mcts-c-factor",
        help = "MCTS: Cpuct growth rate per logarithm of number of nodes."
    )]
    pub mcts_cpuct_factor: Option<f32>,

    #[structopt(
        short = "I",
        long = "--mcts-iters",
        help = "MCTS: number of playouts per playout step"
    )]
    pub mcts_random_playout_iters: Option<u32>,

    #[structopt(long = "--mcts-max-steps", help = "MCTS: max steps per playout")]
    pub mcts_random_playout_max_steps: Option<u32>,

    #[structopt(long = "--mcts-playout-bias", help = "MCTS: random playout bias")]
    pub mcts_random_playout_bias: Option<f32>,

    #[structopt(long = "--mcts-policy-bias", help = "MCTS: policy network bias")]
    pub mcts_policy_bias: Option<f32>,

    #[cfg(feature = "training")]
    #[structopt(long = "--mcts-policy-npz", help = "MCTS: Path to policy .npz file")]
    pub mcts_policy_npz_path: Option<PathBuf>,

    #[structopt(long = "--mcts-use-policy-network", help = "MCTS: Use hard-coded policy network")]
    pub mcts_use_policy_network: bool,

    #[structopt(
        long = "--mcts-use-rule-based-policy",
        help = "MCTS: Use rule-based search as policy"
    )]
    pub mcts_use_rule_based_policy: bool,

    #[structopt(long = "--policy-based-bias", help = "Policy-based: softmax bias")]
    pub policy_based_bias: Option<f32>,

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
        help = "Path to the deck for Player 1. Within the file, the card names areseparated by line break and there is a blank line between character cards and the deck cards."
    )]
    pub player1_deck: Option<PathBuf>,

    #[structopt(
        parse(from_os_str),
        short = "b",
        long = "--player2-deck",
        help = "Path to the deck for Player 2. Within the file, the card names areseparated by line break and there is a blank line between character cards and the deck cards."
    )]
    pub player2_deck: Option<PathBuf>,

    #[structopt(
        short = "R",
        long = "--random-decks",
        help = "Randomize both players characters and decks"
    )]
    pub random_decks: bool,

    #[structopt(
        short = "M",
        long = "--mirror-match",
        help = "Player 2's deck will copy player 1's deck (can be random or specified)"
    )]
    pub mirror_match: bool,
}

#[derive(Debug, StructOpt, Clone)]
pub struct SearchOpts {
    #[structopt(flatten)]
    pub deck: DeckGen,

    #[structopt(short = "s", long = "--steps", help = "Benchmark max steps to play out")]
    pub steps: Option<u32>,

    #[structopt(short = "S", long = "--seed", help = "Random seed for the game states")]
    pub seed: Option<u64>,

    #[structopt(
        long = "--tactical",
        help = "Tactical mode for search (a special mode that ignores cards, Elemental Tuning and dice management)"
    )]
    pub tactical: bool,

    #[structopt(flatten)]
    pub search: SearchConfig,
}

impl DeckGen {
    pub fn get_decks<R: Rng>(&self, rng: &mut R) -> Result<(Decklist, Decklist), std::io::Error> {
        let mut get_deck = move |path: &Option<PathBuf>| -> Result<Decklist, std::io::Error> {
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
        let deck2 = if self.mirror_match {
            deck1.clone()
        } else {
            get_deck(&self.player2_deck)?
        };
        Ok((deck1, deck2))
    }
}

impl SearchOpts {
    pub fn get_decks(&self) -> Result<(Decklist, Decklist), std::io::Error> {
        let mut r = SmallRng::seed_from_u64(self.seed.unwrap_or(100));
        self.deck.get_decks(&mut r)
    }

    pub fn get_standard_game(&self, rng: Option<SmallRng>) -> Result<GameStateWrapper, std::io::Error> {
        let (d1, d2) = self.get_decks()?;
        let rng = rng.unwrap_or_else(|| SmallRng::seed_from_u64(self.seed.unwrap_or(100)));
        let mut game = new_standard_game(&d1, &d2, rng);
        if self.tactical {
            game.convert_to_tactical_search();
        }
        Ok(game)
    }
}

pub enum GenericSearch<S: NondetState = StandardNondetHandlerState> {
    Minimax(MinimaxSearch<GameStateWrapper<S>>),
    MCTS(MCTS<GameStateWrapper<S>>),
    MCTSRuleBasedPolicy(MCTS<GameStateWrapper<S>, gitcg_sim_search::mcts::policy::DefaultEvalPolicy, RuleBasedPuct>),
    MCTSPolicy(MCTS<GameStateWrapper<S>, gitcg_sim_search::mcts::policy::DefaultEvalPolicy, PolicyNetwork>),
    RuleBasedSearch(RuleBasedSearch),
    PolicyBasedSearch(PolicyNetworkBasedSearch<SmallRng>),
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

impl<S: NondetState> GameTreeSearch<GameStateWrapper<S>> for GenericSearch<S> {
    fn search(
        &mut self,
        position: &GameStateWrapper<S>,
        maximize_player: PlayerId,
    ) -> SearchResult<GameStateWrapper<S>> {
        match self {
            Self::Minimax(s) => s.search(position, maximize_player),
            Self::MCTS(s) => s.search(position, maximize_player),
            Self::MCTSRuleBasedPolicy(s) => s.search(position, maximize_player),
            Self::MCTSPolicy(s) => s.search(position, maximize_player),
            Self::RuleBasedSearch(s) => s.search(position, maximize_player),
            Self::PolicyBasedSearch(s) => s.search(position, maximize_player),
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
                        .unwrap_or(gitcg_sim_search::minimax::transposition_table::DEFAULT_SIZE_MB),
                    debug: self.debug,
                    tactical_depth: self.tactical_depth.unwrap_or(TACTICAL_SEARCH_DEPTH),
                    target_round_delta: self.target_round_delta.unwrap_or(TARGET_ROUND_DELTA),
                    static_search_max_iters: self.static_search_iters.unwrap_or(STATIC_SEARCH_MAX_ITERS),
                };
                GenericSearch::Minimax(MinimaxSearch::new(config))
            }
            SearchAlgorithm::MCTS => {
                let config = MCTSConfig {
                    cpuct: self.get_cpuct_config(),
                    tt_size_mb: self.tt_size_mb.unwrap_or(32),
                    limits,
                    parallel,
                    random_playout_iters: self.mcts_random_playout_iters.unwrap_or(10),
                    random_playout_cutoff: self.mcts_random_playout_max_steps.unwrap_or(20),
                    random_playout_bias: self.mcts_random_playout_bias,
                    policy_bias: self.mcts_policy_bias,
                    debug: self.debug,
                };
                if self.mcts_use_rule_based_policy {
                    let selection_policy = Default::default();
                    return GenericSearch::MCTSRuleBasedPolicy(MCTS::new_with_eval_policy_and_selection_policy(
                        config,
                        Default::default(),
                        selection_policy,
                    ));
                }
                if self.mcts_use_policy_network {
                    let selection_policy = PolicyNetwork::new_hard_coded();
                    return GenericSearch::MCTSPolicy(MCTS::new_with_eval_policy_and_selection_policy(
                        config,
                        Default::default(),
                        selection_policy,
                    ));
                }
                #[cfg(feature = "training")]
                if let Some(npz_path) = &self.mcts_policy_npz_path {
                    let selection_policy = PolicyNetwork::from_npz(npz_path).expect("Failed to load .npz.");
                    return GenericSearch::MCTSPolicy(MCTS::new_with_eval_policy_and_selection_policy(
                        config,
                        Default::default(),
                        selection_policy,
                    ));
                }
                GenericSearch::MCTS(MCTS::new(config))
            }
            SearchAlgorithm::PolicyBased => GenericSearch::PolicyBasedSearch(PolicyNetworkBasedSearch::new(
                SmallRng::from_entropy(),
                self.policy_based_bias,
                PolicyNetwork::new_hard_coded(),
            )),
            SearchAlgorithm::RuleBased => GenericSearch::RuleBasedSearch(RuleBasedSearch::new(Default::default())),
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

    pub fn get_cpuct_config(&self) -> CpuctConfig {
        CpuctConfig {
            init: self.mcts_cpuct_init.unwrap_or(CpuctConfig::STANDARD.init),
            base: self.mcts_cpuct_base.unwrap_or(CpuctConfig::STANDARD.base),
            factor: self.mcts_cpuct_factor.unwrap_or(CpuctConfig::STANDARD.factor),
        }
    }
}

impl SearchOpts {
    pub fn make_search<S: NondetState>(&self, parallel: bool, limits: Option<SearchLimits>) -> GenericSearch<S> {
        self.search.make_search(parallel, limits)
    }

    pub fn get_limits(&self) -> Option<SearchLimits> {
        self.search.get_limits()
    }
}
