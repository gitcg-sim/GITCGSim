use std::ops::Add;

mod game_trait;

pub use game_trait::*;

mod game_state_wrapper;
pub use game_state_wrapper::*;

use crate::{data_structures::LinkedList, linked_list, types::game_state::PlayerId};

/// Principal Variation: A sequence of moves known to be best and is used to guide the search to be more efficient at pruning.
#[allow(type_alias_bounds)]
pub type PV<G: Game> = LinkedList<G::Action>;

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchCounter {
    /// Number of states visited through game state advancements.
    pub states_visited: u64,
    #[cfg(detailed_search_stats)]
    /// Number of times the pruning condition has been reached.
    pub beta_prunes: u64,
    #[cfg(detailed_search_stats)]
    /// Number of times ALL nodes have been searched through.
    pub all_nodes: u64,
    #[cfg(detailed_search_stats)]
    /// Zero-window search failures
    pub zws_fails: u64,
    #[cfg(detailed_search_stats)]
    /// Number of times aspiration window fail-high conditions trigger
    pub aw_fail_highs: u64,
    #[cfg(detailed_search_stats)]
    /// Number of times aspiration window fail-low conditions trigger
    pub aw_fail_lows: u64,
    #[cfg(detailed_search_stats)]
    /// Number of aspiration window iterations
    pub aw_iters: u64,
    /// Number of times a board position was being evaluated (zero depth or winner found).
    pub evals: u64,
    /// Number of times there is a transposition table hit
    pub tt_hits: u64,
    /// Last finished depth for iterative deepening
    pub last_depth: u8,
}

impl SearchCounter {
    #[cfg(detailed_search_stats)]
    pub const ZERO: SearchCounter = SearchCounter {
        states_visited: 0,
        beta_prunes: 0,
        all_nodes: 0,
        zws_fails: 0,
        evals: 0,
        aw_fail_highs: 0,
        aw_fail_lows: 0,
        aw_iters: 0,
        tt_hits: 0,
        last_depth: 0,
    };

    #[cfg(not(detailed_search_stats))]
    pub const ZERO: SearchCounter = SearchCounter {
        states_visited: 0,
        evals: 0,
        tt_hits: 0,
        last_depth: 0,
    };

    pub const EVAL: SearchCounter = SearchCounter {
        states_visited: 1,
        evals: 1,
        ..Self::ZERO
    };

    pub const HIT: SearchCounter = SearchCounter {
        tt_hits: 1,
        ..Self::ZERO
    };

    #[inline]
    pub fn add_in_place(&mut self, c: &SearchCounter) {
        self.states_visited += c.states_visited;
        #[cfg(detailed_search_stats)]
        {
            self.beta_prunes += c.beta_prunes;
            self.all_nodes += c.all_nodes;
            self.zws_fails += c.zws_fails;
            self.aw_fail_highs += c.aw_fail_highs;
            self.aw_fail_lows += c.aw_fail_lows;
            self.aw_iters += c.aw_iters;
        }
        self.evals += c.evals;
        self.tt_hits += c.tt_hits;
    }

    pub fn summary(&self, dt_ns: u128) -> String {
        let dt_ms: f64 = 1e-6 * (dt_ns as f64);
        let rate: f64 = (1e-6_f64 * 1e9_f64) * (self.states_visited as f64) / (dt_ns as f64);
        format!("dt={dt_ms:.2}ms rate={rate:.4} Mstates/s")
    }
}

impl Add for SearchCounter {
    type Output = SearchCounter;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let mut a = self;
        a.add_in_place(&rhs);
        a
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchResult<G: Game> {
    /// Principal Variation
    pub pv: PV<G>,
    pub eval: G::Eval,
    pub counter: SearchCounter,
}

impl<G: Game> Default for SearchResult<G> {
    fn default() -> Self {
        SearchResult::new(linked_list![], G::Eval::MIN, Default::default())
    }
}

impl<G: Game> SearchResult<G> {
    #[inline]
    pub(crate) fn new(pv: PV<G>, eval: G::Eval, counter: SearchCounter) -> Self {
        SearchResult { pv, eval, counter }
    }
}

pub trait GameTreeSearch<G: Game> {
    fn search(&mut self, position: &G, maximize_player: PlayerId) -> SearchResult<G>;

    /// Perform search on the position with hidden information taken into account.
    fn search_hidden(&mut self, position: &G, maximize_player: PlayerId) -> SearchResult<G> {
        let mut position1 = position.clone();
        position1.hide_private_information(maximize_player.opposite());
        self.search(&position1, maximize_player)
    }
}

#[derive(Debug, Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SearchLimits {
    pub max_time_ms: Option<u128>,
    pub max_positions: Option<u64>,
}

impl SearchLimits {
    pub fn should_terminate(&self, start_time: std::time::Instant, positions_searched: u64) -> bool {
        if let Some(max_time_ms) = self.max_time_ms {
            let dt = start_time.elapsed();
            return dt.as_millis() >= max_time_ms;
        }
        if let Some(max_positions) = self.max_positions {
            return positions_searched >= max_positions;
        }

        false
    }
}
