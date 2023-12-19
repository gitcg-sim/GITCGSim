use instant::Instant;

use std::{cell::RefCell, ops::ControlFlow, rc::Rc};

use crate::{
    cons, game_tree_search::*, linked_list, minimax::transposition_table::TTKey, transposition_table::CacheTable,
    types::game_state::PlayerId, zobrist_hash::HashValue,
};
use atree::{Arena, Token};
use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng};

#[cfg(not(feature = "no_parallel"))]
use rayon::prelude::*;
use smallvec::SmallVec;

use self::policy::{DefaultEvalPolicy, EvalPolicy, SelectionPolicy, SelectionPolicyContext, UCB1};

pub mod policy;

pub mod proportion;
use proportion::*;

type TTValue = Proportion;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreeDump<T> {
    #[cfg_attr(feature = "serde", serde(rename = "_"))]
    pub value: T,
    #[cfg_attr(feature = "serde", serde(rename = "children"))]
    pub children: Vec<Rc<TreeDump<T>>>,
}

impl<T> TreeDump<T> {
    pub fn new(value: T, children: Vec<Rc<TreeDump<T>>>) -> Self {
        Self { value, children }
    }
}

fn format_ratio(p: Proportion) -> String {
    let r = p.ratio();
    format!("{p} = {:.2}% \u{b1} {:.2}%", 1e2 * r, 1e2 * 2.0 * p.sd())
}

enum IterationEnd {
    WinnerFound { winner: PlayerId, depth: u8 },
    NoChildren,
}

#[derive(Clone)]
pub struct Node<G: Game> {
    pub state: G,
    pub action: Option<G::Action>,
    pub prop: Proportion,
    pub depth: u8,
}

impl<G: Game> std::fmt::Debug for Node<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("state_hash", &self.state.zobrist_hash())
            .field("q", &self.prop.q)
            .field("n", &self.prop.n)
            .field("depth", &self.depth)
            .finish()
    }
}

impl<G: Game> Node<G> {
    #[inline]
    pub fn new(state: G, action: Option<G::Action>) -> Self {
        Self {
            state,
            action,
            prop: Default::default(),
            depth: 0,
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

    fn debug_description(&self, children_count: usize, describe_action: &dyn Fn(G::Action) -> String) -> String {
        let action = self.action;
        let action_part = if let Some(action) = action {
            describe_action(action)
        } else {
            "[Root]".to_string()
        };

        format!(
            "{action_part} ({}), #children = {}, depth={}",
            format_ratio(self.prop),
            children_count,
            self.depth
        )
    }
}

/// Configuration parameters for evaluating the Cpuct factor for the UCT search algorithm.
/// Cpuct(N) = init + factor * log2((N + base) / base)
/// Based on Lc0 Cpuct parameters: https://lczero.org/play/configuration/flags/
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CpuctConfig {
    pub init: f32,
    pub base: f32,
    pub factor: f32,
}

impl CpuctConfig {
    pub const STANDARD: Self = Self {
        init: 2.2,
        base: 18000.0,
        factor: 2.8,
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

#[derive(Debug)]
pub struct MCTS<G: Game, E: EvalPolicy<G> = DefaultEvalPolicy, S: SelectionPolicy<G> = UCB1> {
    pub config: MCTSConfig,
    pub maximize_player: PlayerId,
    pub tree: Arena<Node<G>>,
    pub tt: CacheTable<TTKey, TTValue>,
    pub root: Option<(HashValue, Token)>,
    pub eval_policy: E,
    pub selection_policy: S,
}

impl<G: Game, E: EvalPolicy<G>, S: SelectionPolicy<G> + Default> MCTS<G, E, S> {
    pub fn new_with_eval_policy(config: MCTSConfig, eval_policy: E) -> Self {
        let tree = Arena::<Node<G>>::new();
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
        let tree = Arena::<Node<G>>::new();
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
        let root = Node::new(init, None);
        let (tree, root_token) = Arena::<Node<G>>::with_data(root);
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
            let node = Node::new(next, Some(action));
            token.append(&mut self.tree, node);
            n += 1;
        }
        Ok(n)
    }

    fn select_level(&self, token: Token, tt_hits: Rc<RefCell<u64>>) -> Option<(G::Action, Token)> {
        let node = self.tree.get(token)?;
        node.data.state.to_move()?;
        let is_maximize = node.data.is_maximize(self.maximize_player);
        let policy = &self.selection_policy;
        let best = {
            let Self { config, tt, tree, .. } = &self;
            let parent = &node.data;
            let ctx = SelectionPolicyContext {
                config,
                parent,
                is_maximize,
            };
            let state = policy.uct_parent_factor(&ctx);
            node.children(tree).max_by_key(move |&child| {
                let child_node = &child.data;
                let (ratio, _) = child_node.ratio_with_transposition(is_maximize, tt, tt_hits.clone());
                let uct = policy.uct_child_factor(&ctx, child_node, &state);
                let bandit = ratio + uct;
                (1e6 * bandit) as u32
            })
        };

        best.and_then(|b| b.data.action.map(|a| (a, b.token())))
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
        node: &atree::Node<Node<G>>,
        is_maximize: bool,
        tt_hits: Rc<RefCell<u64>>,
    ) -> Option<&atree::Node<Node<G>>> {
        node.children(&self.tree).max_by_key(|child_node| {
            let ratio = child_node
                .data
                .ratio_with_transposition(is_maximize, &self.tt, tt_hits.clone())
                .0;
            (1e6 * ratio) as u32
        })
    }

    fn get_pv_rec(&self, node: &atree::Node<Node<G>>, tt_hits: Rc<RefCell<u64>>) -> PV<G> {
        if node.is_leaf() {
            return Default::default();
        }
        let is_maximize = node.data.is_maximize(self.maximize_player);
        #[cfg(any())]
        {
            let actions_expected = node.data.state.actions().into_iter().collect::<Vec<_>>();
            let actions_mcts = node
                .children(&self.tree)
                .map(|n| n.data.action.unwrap())
                .collect::<Vec<_>>();
            if actions_expected != actions_mcts {
                println!("---");
                dbg!(&actions_expected);
                dbg!(&actions_mcts);
                println!("get_pv_rec: Children mismatch.");
            }
        }
        let best_node = self
            .get_best_child(node, is_maximize, tt_hits.clone())
            .expect("get_best_child: Must be non-empty");
        let Some(best) = best_node.data.action else {
            return linked_list![];
        };
        #[cfg(any())]
        {
            let state = node.data.state.clone();
            if !state.actions().into_iter().any(|a| a == best) {
                dbg!(&state);
                self.dump_tree(node.token(), 2, &|a| format!("{a:?}"));
                panic!("get_pv: Action is not available: {best:?}");
            }
        }
        cons!(best, self.get_pv_rec(best_node, tt_hits))
    }

    pub fn get_pv(&self, token: Token) -> PV<G> {
        let tt_hits: Rc<RefCell<u64>> = Rc::new(Default::default());
        let res = self.get_pv_rec(self.tree.get(token).expect("get_pv: node must exist"), tt_hits);
        if res.is_empty() {
            let node = self.tree.get(token).expect("get");
            dbg!(&node.data);
            dbg!(&node.children(&self.tree).map(|n| n.data.clone()).collect::<Vec<_>>());
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
                (0..random_playout_iters)
                    .into_par_iter()
                    .map(|_| {
                        let mut rng = thread_rng();
                        let (count, win) = self.random_playout(next, &mut rng);
                        (count, win as u32)
                    })
                    .reduce(|| (0, 0), |(a, b), (c, d)| (a + c, b + d))
            }
        } else {
            (0..random_playout_iters)
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

    pub fn dump_tree(
        &self,
        token: Token,
        max_depth: u8,
        describe_action: &dyn Fn(G::Action) -> String,
    ) -> TreeDump<String> {
        if max_depth == 0 {
            return Default::default();
        }

        let Some(node) = self.tree.get(token) else {
            return Default::default();
        };

        let children_count = node.children(&self.tree).count();
        let desc = node.data.debug_description(children_count, describe_action);
        let max_depth_1 = max_depth - 1;
        let children = node
            .children(&self.tree)
            .map(|child| Rc::new(self.dump_tree(child.token(), max_depth_1, describe_action)))
            .filter(|child| !(child.children.is_empty() && child.value.is_empty()))
            .collect();
        TreeDump::new(desc, children)
    }

    pub fn print_tree(&self, token: Token, depth: u8, max_depth: u8, min_n: u32) {
        if depth > max_depth {
            return;
        }

        let Some(node) = self.tree.get(token) else {
            return;
        };

        fn indent_prefix(indent_depth: u8) -> String {
            let mut s = Default::default();
            if indent_depth == 0 {
                s += "- ";
                return s;
            }
            for _ in 0..indent_depth {
                s += "  ";
            }
            s += "- ";
            s
        }

        let node_part: String = node
            .data
            .debug_description(node.children(&self.tree).count(), &|a| format!("{a:?}"));
        println!("{}{}", indent_prefix(depth), node_part);
        let mut omitted_prop = Proportion::default();
        let mut omitted = 0;
        let mut found = false;
        let mut children: SmallVec<[_; 16]> = node.children(&self.tree).collect();
        let is_maximize = node.data.is_maximize(self.maximize_player);
        children.sort_by_cached_key(|c| (-1e6 * c.data.ratio(is_maximize)) as i32);
        let c = children.len();
        for (i, child) in children.iter().copied().enumerate() {
            let Node { prop, .. } = child.data;
            let n = prop.n;
            if n != 0 && (c <= 1 || depth == 0 || n >= min_n || i == 0) {
                found = depth < max_depth;
                self.print_tree(child.token(), depth + 1, max_depth, min_n);
            } else {
                omitted += 1;
                omitted_prop += prop;
            }
        }

        if found && omitted > 0 {
            println!(
                "{}...[{omitted} omitted] ({})",
                indent_prefix(depth + 1),
                format_ratio(omitted_prop)
            );
        }
    }

    pub fn get_self_play_data_points(
        &self,
        maximize_player: PlayerId,
        min_depth: u8,
        vec: &mut Vec<(G, G::Action, u8)>,
    ) {
        fn traverse<D, F: FnMut(Token, u8)>(tree: &Arena<D>, token: Token, f: &mut F) -> u8 {
            let Some(node) = tree.get(token) else { return 0 };
            let depth = node
                .children_tokens(tree)
                .map(|token| 1 + traverse(tree, token, f))
                .max()
                .unwrap_or_default();
            f(token, depth);
            depth
        }

        let Some((_, root)) = self.root else { return };
        traverse(&self.tree, root, &mut |token, depth| {
            if depth < min_depth {
                return;
            }

            let Some(node) = self.tree.get(token) else { return };
            let is_maximize = node.data.is_maximize(maximize_player);
            if let Some(best_move) = self
                .get_best_child(node, is_maximize, Default::default())
                .and_then(|node| node.data.action)
            {
                vec.push((node.data.state.clone(), best_move, depth));
            }
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

        let counter = SearchCounter {
            states_visited,
            tt_hits: *tt_hits.borrow(),
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

#[cfg(feature = "serde")]
#[cfg(test)]
mod tests {
    use crate::{
        game_tree_search::{Game, GameStateWrapper, GameTreeSearch, SearchLimits, ZobristHashable},
        mcts::{CpuctConfig, MCTSConfig, MCTS},
        types::{game_state::PlayerId, nondet::StandardNondetHandlerState},
    };

    const CONFIG: MCTSConfig = MCTSConfig {
        cpuct: CpuctConfig::STANDARD,
        tt_size_mb: 0,
        parallel: false,
        random_playout_iters: 10,
        random_playout_cutoff: 20,
        random_playout_bias: Some(50.0),
        policy_bias: None,
        debug: false,
        limits: Some(SearchLimits {
            max_time_ms: Some(100),
            max_positions: None,
        }),
    };

    #[test]
    fn test_problematic_state() {
        let json = "{\"game_state\":{\"pending_cmds\":null,\"round_number\":2,\"phase\":{\"ActionPhase\":{\"first_end_round\":\"PlayerFirst\",\"active_player\":\"PlayerSecond\"}},\"players\":[{\"active_char_idx\":0,\"dice\":{\"omni\":0,\"elem\":[0,0,1,0,0,1,0]},\"char_states\":[{\"char_id\":\"Noelle\",\"_hp_and_energy\":65,\"applied\":0,\"flags\":6},{\"char_id\":\"KamisatoAyaka\",\"_hp_and_energy\":10,\"applied\":0,\"flags\":0}],\"status_collection\":{\"responds_to\":512,\"responds_to_triggers\":4,\"responds_to_events\":0,\"_status_entries\":[{\"key\":{\"Character\":[1,\"KamisatoArtSenho\"]},\"state\":{\"_repr\":128}}]},\"hand\":[],\"flags\":1},{\"active_char_idx\":0,\"dice\":{\"omni\":0,\"elem\":[0,0,1,0,0,0,0]},\"char_states\":[{\"char_id\":\"Noelle\",\"_hp_and_energy\":72,\"applied\":0,\"flags\":14},{\"char_id\":\"KamisatoAyaka\",\"_hp_and_energy\":10,\"applied\":0,\"flags\":0}],\"status_collection\":{\"responds_to\":1549,\"responds_to_triggers\":4,\"responds_to_events\":256,\"_status_entries\":[{\"key\":{\"Character\":[1,\"KamisatoArtSenho\"]},\"state\":{\"_repr\":128}},{\"key\":{\"Character\":[0,\"SweepingTime\"]},\"state\":{\"_repr\":2}},{\"key\":{\"Team\":\"FullPlate\"},\"state\":{\"_repr\":2}}]},\"hand\":[\"Starsigns\",\"Starsigns\"],\"flags\":0}],\"log\":{\"enabled\":false,\"events\":[]},\"ignore_costs\":false,\"_incremental_hash\":1270447005094472145,\"_hash\":1270447005094472145},\"nd\":{\"state\":{\"decks\":[{\"deck\":[\"WolfsGravestone\",\"WolfsGravestone\",\"WolfsGravestone\",\"WolfsGravestone\",\"WolfsGravestone\",\"WolfsGravestone\",\"Starsigns\",\"Starsigns\",\"Starsigns\",\"Starsigns\",\"Starsigns\",\"Starsigns\"],\"mask\":1336,\"count\":5},{\"deck\":[\"WolfsGravestone\",\"WolfsGravestone\",\"WolfsGravestone\",\"WolfsGravestone\",\"WolfsGravestone\",\"WolfsGravestone\",\"Starsigns\",\"Starsigns\",\"Starsigns\",\"Starsigns\",\"Starsigns\",\"Starsigns\"],\"mask\":2581,\"count\":5}],\"rng\":[1858280268712277698,16272710452965485635,11711732624276845211,291822442333847434],\"flags\":0}}}";
        let game: GameStateWrapper<StandardNondetHandlerState> = serde_json::from_str(json).unwrap();
        let mut mcts: MCTS<GameStateWrapper<StandardNondetHandlerState>> = MCTS::new(CONFIG);
        let acts0 = game.actions().into_iter().collect::<Vec<_>>();
        mcts.search(&game, PlayerId::PlayerSecond);
        let root = mcts.root.unwrap().1;
        let game1 = mcts.tree.get(root).unwrap().data.state.clone();
        let acts1 = game1.actions().into_iter().collect::<Vec<_>>();
        dbg!(&game);
        dbg!(&game1);
        println!("{:?}", mcts.get_pv(root));
        println!("{:?}", game.zobrist_hash());
        println!("{:?}", game1.zobrist_hash());
        println!("{:?}", acts0);
        println!("{:?}", acts1);
        assert_eq!(acts0, acts1);
    }
}
