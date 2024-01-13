use instant::Instant;

use std::{
    borrow::Borrow,
    cell::RefCell,
    ops::ControlFlow,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use crate::{
    minimax::transposition_table::TTKey, transposition_table::CacheTable, Game, GameTreeSearch, SearchCounter,
    SearchLimits, SearchResult, PV,
};
use atree::{Arena, Token};
use gitcg_sim::{
    cons,
    game_state_wrapper::*,
    linked_list,
    prelude::{HashValue, PlayerId},
};
use gitcg_sim::{
    rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng},
    smallvec::SmallVec,
};

#[cfg(not(feature = "no_parallel"))]
use rayon::prelude::*;

use self::policy::{
    DefaultEvalPolicy, EvalPolicy, SelectionPolicy, SelectionPolicyChildContext, SelectionPolicyContext, UCB1,
};

pub mod policy;

pub mod proportion;
use proportion::*;

pub mod debug;
pub use debug::*;

type TTValue = Proportion;

enum IterationEnd {
    WinnerFound { winner: PlayerId, depth: u8 },
    NoChildren,
}

#[derive(Debug, Clone, Default)]
pub struct NodeStats {
    pub policy: f32,
    pub score: f32,
    pub ratio: f32,
    pub uct: f32,
}

#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub selected_token: Option<atree::Token>,
    pub visits_remaining: u32,
}

pub struct NodeData<G: Game> {
    pub state: G,
    pub action: Option<G::Action>,
    pub prop: Proportion,
    pub depth: u8,
    pub policy_cache: Mutex<SmallVec<[f32; 16]>>,
    pub selection_state: RwLock<SelectionState>,
    /// Keeps track of mutable statistics. New instances constructed only on `NodeData::new`.
    /// Cannot be cloned.
    pub last_stats: Arc<Mutex<NodeStats>>,
}

impl<G: Game> std::fmt::Debug for NodeData<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeData")
            .field("state_hash", &self.state.zobrist_hash())
            .field("q", &self.prop.q)
            .field("n", &self.prop.n)
            .field("depth", &self.depth)
            .field("last_stats", &self.last_stats)
            .finish()
    }
}

impl<G: Game> NodeData<G> {
    #[inline]
    pub fn new(state: G, action: Option<G::Action>) -> Self {
        Self {
            state,
            action,
            prop: Default::default(),
            depth: Default::default(),
            policy_cache: Default::default(),
            selection_state: Default::default(),
            last_stats: Default::default(),
        }
    }

    #[inline]
    fn is_maximize(&self, maximize_player: PlayerId) -> bool {
        self.state.to_move().unwrap_or(maximize_player) == maximize_player
    }

    #[inline]
    fn ratio(&self, is_maximize: bool) -> f32 {
        if is_maximize {
            self.prop.ratio()
        } else {
            self.prop.complement().ratio()
        }
    }

    #[inline]
    fn ratio_with_transposition(
        &self,
        is_maximize: bool,
        tt: &CacheTable<TTKey, TTValue>,
        tt_hits: Rc<RefCell<u64>>,
    ) -> (f32, u32) {
        let p0 = self.prop;
        let p1 = self.lookup_tt(tt, tt_hits).unwrap_or_default();
        let p = if is_maximize { p0 + p1 } else { (p0 + p1).complement() };
        (p.ratio(), p.n)
    }

    #[inline]
    fn lookup_tt(&self, tt: &CacheTable<TTKey, TTValue>, tt_hits: Rc<RefCell<u64>>) -> Option<Proportion> {
        let Some(res) = tt.get(&TTKey(self.state.zobrist_hash())) else {
            return None;
        };
        *tt_hits.borrow_mut() += 1;
        Some(res)
    }
}

/// Configuration parameters for evaluating the Cpuct factor for the UCT search algorithm.
/// Cpuct(N) = init + factor * log2((N + base) / base)
/// Based on Lc0 Cpuct parameters: <https://lczero.org/play/configuration/flags/>
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CpuctConfig {
    pub init: f32,
    pub base: f32,
    pub factor: f32,
}

impl CpuctConfig {
    pub const STANDARD: Self = Self {
        init: 4.0,
        base: 20000.0,
        factor: 4.0,
    };

    #[inline(always)]
    pub fn cpuct(&self, n: f32) -> f32 {
        let Self { init, factor: k, base } = *self;
        init + if k >= 0.0 { f32::log2((n + base) / base) } else { 0.0 }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MCTSConfig {
    pub cpuct: CpuctConfig,
    pub tt_size_mb: u32,
    pub parallel: bool,
    pub random_playout_iters: u32,
    pub random_playout_cutoff: u32,
    /// Random playout bias, None to disable bias
    pub random_playout_bias: Option<f32>,
    pub policy_bias: Option<f32>,
    pub debug: bool,
    pub limits: Option<SearchLimits>,
}

impl MCTSConfig {
    #[inline]
    pub(crate) fn policy_softmax(&self, v: f32) -> f32 {
        if let Some(a) = self.policy_bias {
            (v * a).exp().clamp(1e-2, 1e3)
        } else {
            1e-2 + v
        }
    }
}

#[cfg(feature = "training")]
#[derive(Debug)]
pub struct SelfPlayDataPoint<G: Game> {
    pub state: G,
    pub action_weights: Vec<(G::Action, f32)>,
    pub depth: u8,
}

#[derive(Debug)]
pub struct MCTS<G: Game, E: EvalPolicy<G> = DefaultEvalPolicy, S: SelectionPolicy<G> = UCB1> {
    pub config: MCTSConfig,
    pub maximize_player: PlayerId,
    pub tree: Arena<NodeData<G>>,
    pub tt: CacheTable<TTKey, TTValue>,
    pub root: Option<(HashValue, Token)>,
    pub eval_policy: E,
    pub selection_policy: S,
}

impl<G: Game, E: EvalPolicy<G>, S: SelectionPolicy<G> + Default> MCTS<G, E, S> {
    pub fn new_with_eval_policy(config: MCTSConfig, eval_policy: E) -> Self {
        let tree = Arena::<NodeData<G>>::new();
        Self {
            config,
            tree,
            maximize_player: PlayerId::PlayerFirst,
            tt: CacheTable::new(config.tt_size_mb as usize),
            root: None,
            eval_policy,
            selection_policy: Default::default(),
        }
    }
}

impl<G: Game, E: EvalPolicy<G>, S: SelectionPolicy<G>> MCTS<G, E, S> {
    pub fn new_with_eval_policy_and_selection_policy(config: MCTSConfig, eval_policy: E, selection_policy: S) -> Self {
        let tree = Arena::<NodeData<G>>::new();
        Self {
            config,
            tree,
            maximize_player: PlayerId::PlayerFirst,
            tt: CacheTable::new(config.tt_size_mb as usize),
            root: None,
            eval_policy,
            selection_policy,
        }
    }
}

impl<G: Game, E: EvalPolicy<G> + Default> MCTS<G, E> {
    pub fn new(config: MCTSConfig) -> Self {
        Self::new_with_eval_policy(config, Default::default())
    }
}

impl<G: Game, E: EvalPolicy<G>, S: SelectionPolicy<G>> MCTS<G, E, S> {
    fn init(&mut self, init: G, maximize_player: PlayerId) -> Token {
        let hash = init.zobrist_hash();
        let root = NodeData::new(init, None);
        let (tree, root_token) = Arena::<NodeData<G>>::with_data(root);
        self.tree = tree;
        self.maximize_player = maximize_player;
        self.tt.clear();
        self.root = Some((hash, root_token));
        root_token
    }

    fn expand(&mut self, token: Token) -> Result<u64, Option<PlayerId>> {
        let Some(current) = self.tree.get(token).map(|x| &x.data.state) else {
            return Err(None);
        };
        if let Some(winner) = current.winner() {
            return Err(Some(winner));
        };

        let current = current.clone();
        let actions = current.actions();
        let mut n = 0;
        for action in actions {
            let mut next = current.clone();
            next.advance(action).unwrap();
            let node = NodeData::new(next, Some(action));
            token.append(&mut self.tree, node);
            n += 1;
        }
        Ok(n)
    }

    fn select_level(&self, token: Token, tt_hits: Rc<RefCell<u64>>) -> Option<(G::Action, Token)> {
        let parent_node = self.tree.get(token)?;
        let parent = &parent_node.data;
        parent.state.to_move()?;
        {
            let sel = parent_node
                .data
                .selection_state
                .try_read()
                .expect("select_level: retrieve selection");
            if sel.visits_remaining > 0 {
                let token = sel.selected_token.expect("selected_token");
                let action = self.tree.get(token).expect("get").data.action.expect("get: action");
                return Some((action, token));
            }
        }

        let is_maximize = parent.is_maximize(self.maximize_player);
        let policy = &self.selection_policy;
        let Self { config, tt, tree, .. } = &self;
        let ctx = SelectionPolicyContext {
            config,
            parent,
            is_maximize,
        };
        let children = parent_node.children(tree).collect::<SmallVec<[_; 16]>>();
        let get_children = || {
            children
                .iter()
                .copied()
                .map(|child| child.data.action.expect("child node action must exist"))
                .collect()
        };
        let state = &policy.on_parent(&ctx, get_children);
        let policy_values = {
            let mut policy_cache = parent.policy_cache.lock().expect("policy_cache unlock");
            if policy_cache.is_empty() && !children.is_empty() {
                let mut tot = 0.0;
                *policy_cache = children
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(index, child_node)| {
                        let child = &child_node.data;
                        let child_ctx = SelectionPolicyChildContext { index, child, state };
                        let val = policy.policy(&ctx, &child_ctx);
                        tot += val;
                        val
                    })
                    .collect();
                if tot >= 0.0 {
                    for v in policy_cache.iter_mut() {
                        *v /= tot;
                    }
                }
            } else if policy_cache.len() != children.len() {
                panic!("non-zero number of children changed");
            }
            policy_cache
        };
        let (mut best, mut best_score, mut second_best_score) = (None, f32::MIN, None);
        for (index, child_node) in children.iter().copied().enumerate() {
            let child = &child_node.data;
            let child_ctx = SelectionPolicyChildContext { index, child, state };
            let (ratio, _) = child.ratio_with_transposition(is_maximize, tt, tt_hits.clone());
            let policy_value = policy_values[index];
            let uct = policy.uct_child(&ctx, &child_ctx, policy_value);
            let score = ratio + uct;
            if let Ok(mut st) = child.last_stats.lock() {
                st.policy = policy_value;
                st.ratio = ratio;
                st.uct = uct;
                st.score = score;
            }

            if score >= best_score {
                second_best_score = Some(best_score);
                best_score = score;
                best = Some(child_node);
            }
        }
        let best = best?;
        let best_action = best.data.action.expect("best.data.action");
        {
            let mut sel = parent_node
                .data
                .selection_state
                .try_write()
                .expect("select_level: get selection");
            const MAX_VISITS: u32 = 100;
            // Estimate the number of visits for the 2nd best score to overtake the best score.
            // P(Q - x, N) + Puct(N0 + x) - second_best_score = 0
            // Linear approximation at x=0, P(Q - x, N) -> P(Q, N) - x/N and Puct(N0 + x) -> Puct(N0)
            // -x / N + (Q / n) + Puct(N0) - second_best_score + O(n^2) = 0
            // (score - second_best_score) - x / N = 0
            // x = N * (score - second_best_score)

            sel.visits_remaining = match second_best_score {
                None => MAX_VISITS,
                Some(second_best_score) => {
                    let delta = best_score - second_best_score;
                    let n = (best.data.prop.n + 1) as f32;
                    // Multiply by 0.7 to further reduce the estimate
                    ((0.7 * delta * n) as u32).max(1).min(MAX_VISITS)
                }
            };
            sel.selected_token = Some(best.token());
        }
        Some((best_action, best.token()))
    }

    fn select(&self, token: Token, path: &mut Vec<Token>, tt_hits: Rc<RefCell<u64>>) -> Token {
        if let Some((_, token1)) = self.select_level(token, tt_hits.clone()) {
            path.push(token1);
            self.select(token1, path, tt_hits)
        } else {
            token
        }
    }

    fn get_best_child(
        &self,
        node: &atree::Node<NodeData<G>>,
        is_maximize: bool,
        tt_hits: Rc<RefCell<u64>>,
    ) -> Option<&atree::Node<NodeData<G>>> {
        node.children(&self.tree).max_by_key(|child_node| {
            let ratio = child_node
                .data
                .ratio_with_transposition(is_maximize, &self.tt, tt_hits.clone())
                .0;
            (1e8 * ratio) as u32
        })
    }

    fn get_pv_rec(&self, node: &atree::Node<NodeData<G>>, tt_hits: Rc<RefCell<u64>>) -> PV<G> {
        if node.is_leaf() {
            return Default::default();
        }
        let is_maximize = node.data.is_maximize(self.maximize_player);
        let best_node = self
            .get_best_child(node, is_maximize, tt_hits.clone())
            .expect("get_best_child: Must be non-empty");
        let Some(best) = best_node.data.action else {
            return linked_list![];
        };
        cons!(best, self.get_pv_rec(best_node, tt_hits))
    }

    pub fn get_pv(&self, token: Token) -> PV<G> {
        let tt_hits: Rc<RefCell<u64>> = Rc::new(Default::default());
        let res = self.get_pv_rec(self.tree.get(token).expect("get_pv: node must exist"), tt_hits);
        if res.is_empty() {
            let node = self.tree.get(token).expect("get");
            dbg!(&node.data);
            dbg!(&node.children(&self.tree).map(|n| &n.data).collect::<Vec<_>>());
            panic!("get_pv: empty");
        } else {
            res
        }
    }

    fn random_playout<R: Rng>(&self, token: Token, rng: &mut R) -> (u64, bool) {
        let mut count = 0;
        let node = self.tree.get(token).unwrap();
        let mut game = node.data.state.clone();
        for _ in 0..self.config.random_playout_cutoff {
            if game.winner().is_some() {
                break;
            }

            let acts = game.actions();
            if let Some(bias) = self.config.random_playout_bias {
                let pairs = game.action_weights(&acts);
                let weights = pairs
                    .iter()
                    .map(|(_, x)| (x * bias).exp().clamp(1e-2, 1e2))
                    .collect::<SmallVec<[_; 16]>>();
                let Ok(dist) = WeightedIndex::new(weights) else {
                    panic!("MCTS::random_playout: Invalid weights: {:?}", pairs);
                };
                let action = pairs[dist.sample(rng)].0;
                game.advance(action).unwrap();
            } else {
                let acts = acts.into_iter().collect::<SmallVec<[_; 8]>>();
                let action = acts[rng.gen_range(0..acts.len())];
                game.advance(action).unwrap();
            }
            count += 1;
        }
        let winner = game.winner().unwrap_or_else(|| {
            if game.eval(self.maximize_player) > Default::default() {
                self.maximize_player
            } else {
                self.maximize_player.opposite()
            }
        });
        (count, winner == self.maximize_player)
    }

    fn backpropagate(&mut self, path: Vec<Token>, dprop: Proportion) {
        let tree = &mut self.tree;
        let n = path.len() as u8;
        let mut d = n;
        for token in path.iter().copied() {
            let data = &mut tree.get_mut(token).unwrap().data;
            let key = TTKey(data.state.zobrist_hash());
            data.prop += dprop;
            data.depth = d;
            d -= 1;
            let prop0: Proportion = self.tt.get(&key).unwrap_or_default();
            let n0 = prop0.n;
            self.tt.replace_if(&key, prop0 + dprop, |pt| pt.n <= n0);
            let mut sel = data
                .selection_state
                .try_write()
                .expect("backpropagate: lock selection_state");
            sel.visits_remaining = sel.visits_remaining.saturating_sub(1);
        }
    }

    fn iteration(&mut self, root: Token, tt_hits: Rc<RefCell<u64>>) -> ControlFlow<IterationEnd, u64> {
        let random_playout_iters = self.config.random_playout_iters;
        let mut path = Vec::with_capacity(8);
        if let Some(winner) = self
            .tree
            .get(root)
            .expect("iteration: root must exist")
            .data
            .state
            .winner()
        {
            return ControlFlow::Break(IterationEnd::WinnerFound { winner, depth: 0 });
        }

        path.push(root);
        let selected = self.select(root, &mut path, tt_hits.clone());
        let selected_data = &self.tree.get(selected).expect("iteration: selected must exist").data;
        let (expand_states_visited, next) = if let Some(winner) = selected_data.state.winner() {
            if !path.is_empty() {
                (0, path[path.len() - 1])
            } else {
                return ControlFlow::Break(IterationEnd::WinnerFound {
                    winner,
                    depth: selected_data.depth,
                });
            }
        } else {
            let expand_states_visited = self.expand(selected).unwrap_or(0);
            let Some((_, selected_token)) = self.select_level(selected, tt_hits) else {
                return ControlFlow::Break(IterationEnd::NoChildren);
            };
            (expand_states_visited, selected_token)
        };
        let no_parallel = cfg!(feature = "no_parallel");
        let (states_visited, wins): (u64, u32) = if !no_parallel && self.config.parallel {
            #[cfg(feature = "no_parallel")]
            {
                unreachable!("#[cfg(feature = \"no_parallel\")]")
            }

            #[cfg(not(feature = "no_parallel"))]
            {
                (0..random_playout_iters.max(1))
                    .into_par_iter()
                    .map(|_| {
                        let mut rng = thread_rng();
                        let (count, win) = self.random_playout(next, &mut rng);
                        (count, win as u32)
                    })
                    .reduce(|| (0, 0), |(a, b), (c, d)| (a + c, b + d))
            }
        } else {
            (0..random_playout_iters.max(1))
                .map(|_| {
                    let mut rng = thread_rng();
                    let (count, win) = self.random_playout(next, &mut rng);
                    (count, win as u32)
                })
                .fold((0, 0), |(a, b), (c, d)| (a + c, b + d))
        };
        self.backpropagate(path, (wins, random_playout_iters).into());

        let counter = 1 + expand_states_visited + states_visited;
        ControlFlow::Continue(counter)
    }

    #[cfg(feature = "training")]
    pub fn get_self_play_policy_data_points<
        FnCheck: Fn(u8, usize) -> bool,
        FnPush: FnMut(SelfPlayDataPoint<G>) -> ControlFlow<()>,
    >(
        &self,
        maximize_player: PlayerId,
        should_include: FnCheck,
        mut push_data_point: FnPush,
    ) {
        fn traverse<D, F: FnMut(Token, u8) -> ControlFlow<()>>(
            tree: &Arena<D>,
            token: Token,
            f: &mut F,
        ) -> ControlFlow<(), u8> {
            let Some(node) = tree.get(token) else {
                return ControlFlow::Continue(0);
            };
            let mut depth = 0;
            for token in node.children_tokens(tree) {
                let d = traverse(tree, token, f)?;
                let d1 = 1 + d;
                if d1 > depth {
                    depth = d1;
                }
            }
            if let ControlFlow::Break(_) = f(token, depth) {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(depth)
            }
        }

        let tree = &self.tree;
        let tt = &self.tt;
        let Some((_, root)) = self.root else { return };
        let pv_depth = self.get_pv(root).len();
        let _ = traverse(&self.tree, root, &mut |token, depth| {
            let node = tree.get(token).expect("get");
            if !should_include(depth, pv_depth) {
                return ControlFlow::Continue(());
            }

            let is_maximize = node.data.is_maximize(maximize_player);
            let action_weights = node
                .children(tree)
                .map(|child| {
                    (
                        child.data.action.unwrap(),
                        child
                            .data
                            .ratio_with_transposition(is_maximize, tt, Default::default())
                            .0,
                    )
                })
                .collect::<Vec<_>>();
            let state = node.data.state.clone();
            push_data_point(SelfPlayDataPoint {
                state,
                action_weights,
                depth,
            })
        });
    }
}

impl<G: Game, E: EvalPolicy<G>, S: SelectionPolicy<G>> GameTreeSearch<G> for MCTS<G, E, S> {
    fn search(&mut self, position: &G, maximize_player: PlayerId) -> SearchResult<G> {
        if position.winner().is_some() {
            return Default::default();
        }

        let time_limit_ms = self.config.limits.and_then(|l| l.max_time_ms).unwrap_or(600_000);
        let states_limit = self.config.limits.and_then(|l| l.max_positions).unwrap_or(u64::MAX);
        let t0 = Instant::now();
        let mut states_visited = 0;
        let tt_hits = Rc::new(RefCell::new(0u64));
        let root = self.init(position.clone(), maximize_player);
        let mut last_print = t0;
        'iter: loop {
            for _ in 0..10 {
                let dn = match self.iteration(root, tt_hits.clone()) {
                    ControlFlow::Continue(dn) => dn,
                    ControlFlow::Break(IterationEnd::WinnerFound { winner, depth }) => {
                        println!("winner found {winner} {depth}");
                        break 'iter;
                    }
                    ControlFlow::Break(IterationEnd::NoChildren) => {
                        panic!("search: iteration failed")
                    }
                };
                states_visited += dn;
                if states_visited >= states_limit {
                    break 'iter;
                }
                if t0.elapsed().as_millis() >= time_limit_ms {
                    break 'iter;
                }
            }
            if self.config.debug && last_print.elapsed().as_millis() >= 500 {
                last_print = Instant::now();
                let pv = self.get_pv(root);
                let rate = (states_visited as f64) / (t0.elapsed().as_micros() as f64);
                println!(
                    "  states_visited={states_visited:8}, PV={:?} rate={:.4}Mstates/s",
                    pv.into_iter().copied().collect::<Vec<_>>(),
                    rate
                );
            }
        }

        let tt_hits_borrow: &RefCell<u64> = Rc::borrow(&tt_hits);
        let ref_tt_hits = tt_hits_borrow.try_borrow().unwrap();
        let counter = SearchCounter {
            states_visited,
            tt_hits: *ref_tt_hits,
            ..Default::default()
        };
        let pv = self.get_pv(root);
        if self.config.debug {
            self.print_tree(root, 0, 2, 40 * self.config.random_playout_iters);
            println!("PV = {:?}", pv.into_iter().copied().collect::<Vec<_>>());
        }
        SearchResult {
            pv,
            eval: Default::default(),
            counter,
        }
    }
}
