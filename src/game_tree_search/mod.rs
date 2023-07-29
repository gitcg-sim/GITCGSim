use std::ops::Add;

mod game_trait;

pub use game_trait::*;

mod game_state_wrapper;
pub use game_state_wrapper::*;
use serde::{Deserialize, Serialize};

use crate::{cons, data_structures::LinkedList, linked_list, types::game_state::PlayerId};

/// Principal Variation: A sequence of moves known to be best and is used to guide the search to be more efficient at pruning.
#[allow(type_alias_bounds)]
pub type PV<G: Game> = LinkedList<G::Action>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct SearchCounter {
    /// Number of states visited through game state advancements.
    pub states_visited: u64,
    /// Number of times the pruning condition has been reached.
    pub beta_prunes: u64,
    /// Number of times ALL nodes have been searched through.
    pub all_nodes: u64,
    /// Zero-window search failures
    pub zws_fails: u64,
    /// Number of times a board position was being evaluated (zero depth or winner found).
    pub evals: u64,
    /// Number of times aspiration window fail-high conditions trigger
    pub aw_fail_highs: u64,
    /// Number of times aspiration window fail-low conditions trigger
    pub aw_fail_lows: u64,
    /// Number of aspiration window iterations
    pub aw_iters: u64,
    /// Number of times there is a transposition table hit
    pub tt_hits: u64,
}

impl SearchCounter {
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
        self.beta_prunes += c.beta_prunes;
        self.all_nodes += c.all_nodes;
        self.zws_fails += c.zws_fails;
        self.evals += c.evals;
        self.aw_fail_highs += c.aw_fail_highs;
        self.aw_fail_lows += c.aw_fail_lows;
        self.aw_iters += c.aw_iters;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult<G: Game> {
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

    #[inline]
    pub(crate) fn negate(&self) -> Self {
        SearchResult {
            pv: self.pv.clone(),
            eval: -self.eval,
            counter: self.counter,
        }
    }

    #[inline]
    pub(crate) fn add_input_and_increment_counter(&self, input: G::Action) -> Self {
        let mut counter = self.counter;
        counter.states_visited += 1;
        SearchResult {
            pv: cons!(input, self.pv.clone()),
            counter,
            eval: self.eval.plus_one_turn(),
        }
    }

    #[inline]
    pub fn update(&mut self, other: &Self) {
        if other.eval > self.eval {
            if !other.pv.is_empty() {
                self.pv = other.pv.clone();
            }
            self.eval = other.eval;
        }
        self.counter.add_in_place(&other.counter);
    }
}

pub trait GameTreeSearch<G: Game> {
    fn search(&mut self, position: &G, maximize_player: PlayerId) -> SearchResult<G>;
}
