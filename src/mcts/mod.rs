use instant::Instant;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

use crate::{
    cons,
    game_tree_search::*,
    linked_list,
    minimax::transposition_table::{TTEntry, TTFlag, TTKey, TTPin, TT},
    types::game_state::PlayerId,
    zobrist_hash::HashValue,
};
use atree::{Arena, Token};
use rand::{thread_rng, Rng};

#[cfg(not(feature = "no_parallel"))]
use rayon::prelude::*;
use smallvec::SmallVec;

type TTValue = (u32, u32);

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TreeDump<T> {
    #[serde(rename = "_")]
    pub value: T,
    #[serde(rename = "children")]
    pub children: Vec<Rc<TreeDump<T>>>,
}

impl<T> TreeDump<T> {
    pub fn new(value: T, children: Vec<Rc<TreeDump<T>>>) -> Self {
        Self { value, children }
    }
}

fn format_ratio(q: u32, n: u32) -> String {
    let r = ((q + 1) as f32) / ((n + 2) as f32);
    let sd = f32::sqrt(r * (1.0 - r)) / f32::sqrt(n as f32);
    format!("{q}/{n} = {:.2}% \u{b1} {:.2}%", 1e2 * r, 1e2 * 2.0 * sd)
}

pub struct Node<G: Game> {
    pub state: G,
    pub action: Option<G::Action>,
    pub q: u32,
    pub n: u32,
    pub depth: u8,
    pub init_prior: Option<f32>,
}

impl<G: Game> std::fmt::Debug for Node<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node").field("q", &self.q).field("n", &self.n).finish()
    }
}

impl<G: Game> Node<G> {
    #[inline]
    pub fn new(state: G, action: Option<G::Action>) -> Self {
        Self {
            state,
            action,
            q: 0,
            n: 0,
            depth: 0,
            init_prior: None,
        }
    }

    #[inline]
    fn is_maximize(&self, maximize_player: PlayerId) -> bool {
        self.state.to_move().unwrap_or(maximize_player) == maximize_player
    }

    #[inline]
    fn ratio(&self, is_maximize: bool) -> f32 {
        let n = self.n + 2;
        let q = if is_maximize { self.q + 1 } else { n - (self.q + 1) };
        (q as f32) / (n as f32)
    }

    #[inline]
    fn prior(&self, tt: TTPin<TTValue, G::Action>, tt_hits: Rc<RefCell<u64>>) -> Option<f32> {
        let Some(res) = tt.get(&TTKey(self.state.zobrist_hash())) else {
            return None
        };
        *tt_hits.borrow_mut() += 1;

        let (q, n) = res.value;
        Some((q as f32) / (n as f32))
    }

    fn debug_description(&self, children_count: usize, describe_action: &dyn Fn(G::Action) -> String) -> String {
        let action = self.action;
        let action_part = if let Some(action) = action {
            describe_action(action)
        } else {
            "[Root]".to_string()
        };

        format!(
            "{action_part} ({}), #children = {}, depth={}{}",
            format_ratio(self.q, self.n),
            children_count,
            self.depth,
            self.init_prior
                .map(|x| format!(", init_prior={:2}%", x * 1e2))
                .unwrap_or_default()
        )
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct MCTSConfig {
    pub c: f32,
    pub tt_size_mb: u32,
    pub parallel: bool,
    pub random_playout_iters: u32,
    pub random_playout_cutoff: u32,
    pub debug: bool,
    pub limits: Option<SearchLimits>,
}

#[derive(Debug)]
pub struct MCTS<G: Game> {
    pub config: MCTSConfig,
    pub maximize_player: PlayerId,
    pub tree: Arena<Node<G>>,
    pub tt: TT<TTValue, G::Action>,
    pub root: Option<(HashValue, Token)>,
}

impl<G: Game> MCTS<G> {
    pub fn new(config: MCTSConfig) -> Self {
        let tree = Arena::<Node<G>>::new();
        Self {
            config,
            tree,
            maximize_player: PlayerId::PlayerFirst,
            tt: TT::new(config.tt_size_mb),
            root: None,
        }
    }

    fn init(&mut self, init: G, maximize_player: PlayerId, tt_hits: Rc<RefCell<u64>>) -> Token {
        let hash = init.zobrist_hash();
        if maximize_player == self.maximize_player {
            if let Some((root_hash, root_token)) = self.root {
                if root_hash == hash {
                    return root_token;
                }
            }
        }
        let mut root = Node::new(init, None);
        root.init_prior = root.prior(self.tt.pin(), tt_hits);
        let (tree, root_token) = Arena::<Node<G>>::with_data(root);
        self.tree = tree;
        self.maximize_player = maximize_player;
        self.root = Some((hash, root_token));
        root_token
    }

    fn expand(&mut self, token: Token, tt_hits: Rc<RefCell<u64>>) -> Result<u64, Option<PlayerId>> {
        let Some(current) = self.tree.get(token).map(|x| &x.data.state) else {
            return Err(None)
        };
        if let Some(winner) = current.winner() {
            return Err(Some(winner));
        };

        let current = current.clone();
        let mut n = 0;
        for action in current.actions() {
            let mut next = current.clone();
            next.advance(action).unwrap();
            let mut node = Node::new(next, Some(action));
            node.init_prior = node.prior(self.tt.pin(), tt_hits.clone());
            token.append(&mut self.tree, node);
            n += 1;
        }
        Ok(n)
    }

    fn select_level(&self, token: Token, tt_hits: Rc<RefCell<u64>>) -> Option<(G::Action, Token)> {
        let node = self.tree.get(token)?;
        node.data.state.to_move()?;
        let c = self.config.c;
        let n0 = node.data.n;
        let ln = 2f32 * f32::ln((n0 + 1) as f32);
        //let ln = f32::sqrt(n0 as f32);
        let is_maximize = node.data.is_maximize(self.maximize_player);
        let best = node.children(&self.tree).into_iter().max_by_key(move |&child| {
            let n = child.data.n;
            let ratio = child.data.ratio(is_maximize);
            let sr = f32::sqrt(ln / ((1 + n) as f32));
            let prior = child.data.prior(self.tt.pin(), tt_hits.clone()).unwrap_or(0.5);
            let bandit = ratio + c * prior * sr;
            (1e6 * bandit) as u32
        });

        best.and_then(|b| b.data.action.map(|a| (a, b.token())))
    }

    pub fn select(&self, token: Token, path: &mut Vec<Token>, tt_hits: Rc<RefCell<u64>>) -> Token {
        if let Some((_, token1)) = self.select_level(token, tt_hits.clone()) {
            path.push(token1);
            self.select(token1, path, tt_hits)
        } else {
            token
        }
    }

    pub fn get_pv(&self, token: Token) -> PV<G> {
        let Some(node) = self.tree.get(token) else {
            return linked_list![]
        };
        let is_maximize = node.data.is_maximize(self.maximize_player);
        node.children(&self.tree)
            .into_iter()
            .max_by_key(|&child| {
                let ratio = child.data.ratio(is_maximize);
                (1e6 * ratio) as u32
            })
            .and_then(|best| best.data.action.map(|a| (a, best.token())))
            .map(|(act, token1)| cons!(act, self.get_pv(token1)))
            .unwrap_or(linked_list![])
    }

    pub fn evaluate(&self, token: Token) -> G::Eval {
        let node = self.tree.get(token).unwrap();
        node.data.state.eval(self.maximize_player)
    }

    fn random_playout<R: Rng>(&self, token: Token, rng: &mut R) -> (u64, bool) {
        fn get_cut<R: Rng>(rng: &mut R, mut n: usize, iters: u8) -> usize {
            for _ in 0..iters {
                if n <= 1 {
                    break;
                }
                n = rng.gen_range(0..=n);
            }
            std::cmp::max(1, n)
        }

        let mut count = 0;
        let node = self.tree.get(token).unwrap();
        let mut game = node.data.state.clone();
        for _ in 0..self.config.random_playout_cutoff {
            if game.winner().is_some() {
                break;
            }

            // let acts = game.actions().into_iter().collect::<SmallVec<[_; 8]>>();
            // let action = acts[rng.gen_range(0..acts.len())];
            // game.advance(action).unwrap();
            let mut acts = game.actions();
            if false {
                game.move_ordering(&Default::default(), &mut acts);
                let acts = acts.into_iter().collect::<SmallVec<[_; 8]>>();
                let cut = get_cut(rng, acts.len(), 1);
                let action = acts[rng.gen_range(0..cut)];
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

    fn backpropagate(&mut self, path: Vec<Token>, dq: u32, dn: u32) {
        let tree = &mut self.tree;
        let mut n = path.len() as u8;
        for token in path {
            let data = &mut tree.get_mut(token).unwrap().data;
            let pin = self.tt.pin();
            data.q += dq;
            data.n += dn;
            data.depth = n;
            let entry = TTEntry::new(TTFlag::Exact, data.depth, (data.q, data.n), linked_list![]);
            pin.insert(TTKey(data.state.zobrist_hash()), entry);
            n -= 1;
        }
    }

    pub fn iteration(&mut self, root: Token, tt_hits: Rc<RefCell<u64>>) -> Option<u64> {
        let random_playout_iters = self.config.random_playout_iters;
        let mut path = Vec::with_capacity(8);
        if self.tree.get(root).and_then(|x| x.data.state.winner()).is_some() {
            //println!("winner found at root");
            return None;
        }

        path.push(root);
        let selected = self.select(root, &mut path, tt_hits.clone());
        let (expand_states_visited, next) = if self.tree.get(selected).unwrap().data.state.winner().is_some() {
            if !path.is_empty() {
                (0, path[path.len() - 1])
            } else {
                println!("iteration: path.is_empty()");
                return None;
            }
        } else {
            let expand_states_visited = self.expand(selected, tt_hits.clone()).unwrap_or(0);
            (expand_states_visited, self.select_level(selected, tt_hits)?.1)
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
                .into_iter()
                .map(|_| {
                    let mut rng = thread_rng();
                    let (count, win) = self.random_playout(next, &mut rng);
                    (count, win as u32)
                })
                .fold((0, 0), |(a, b), (c, d)| (a + c, b + d))
        };
        self.backpropagate(path, wins, random_playout_iters);

        let counter = 1 + expand_states_visited + states_visited;
        Some(counter)
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
            return Default::default()
        };

        let children_count = node.children(&self.tree).count();
        let desc = node.data.debug_description(children_count, describe_action);
        let max_depth_1 = max_depth - 1;
        let children = node
            .children(&self.tree)
            .into_iter()
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
        let (mut omitted_q, mut omitted_n) = (0, 0);
        let mut omitted = 0;
        let mut found = false;
        let mut children: SmallVec<[_; 16]> = node.children(&self.tree).collect();
        let is_maximize = node.data.is_maximize(self.maximize_player);
        children.sort_by_cached_key(|c| (-1e6 * c.data.ratio(is_maximize)) as i32);
        let c = children.len();
        for (i, child) in children.iter().copied().enumerate() {
            let Node { n, q, .. } = child.data;
            if n != 0 && (c <= 1 || depth == 0 || n >= min_n || i == 0) {
                found = depth < max_depth;
                self.print_tree(child.token(), depth + 1, max_depth, min_n);
            } else {
                omitted += 1;
                omitted_q += q;
                omitted_n += n;
            }
        }

        if found && omitted > 0 {
            println!(
                "{}...[{omitted} omitted] ({})",
                indent_prefix(depth + 1),
                format_ratio(omitted_q, omitted_n)
            );
        }
    }
}

impl<G: Game> GameTreeSearch<G> for MCTS<G> {
    fn search(&mut self, position: &G, maximize_player: PlayerId) -> SearchResult<G> {
        if position.winner().is_some() {
            return Default::default();
        }

        let time_limit_ms = self.config.limits.and_then(|l| l.max_time_ms).unwrap_or(600_000);
        let states_limit = self.config.limits.and_then(|l| l.max_positions).unwrap_or(u64::MAX);
        let t0 = Instant::now();
        let mut states_visited = 0;
        let tt_hits = Rc::new(RefCell::new(0u64));
        let root = self.init(position.clone(), maximize_player, tt_hits.clone());
        'iter: loop {
            for _ in 0..10 {
                let Some(dn) = self.iteration(root, tt_hits.clone()) else {
                    break 'iter;
                };
                states_visited += dn;
                if states_visited >= states_limit {
                    break 'iter;
                }
                if t0.elapsed().as_millis() >= time_limit_ms {
                    break 'iter;
                }
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
            println!("PV = {:?}", pv.clone().collect::<Vec<_>>());
        }
        SearchResult {
            pv,
            eval: Default::default(),
            counter,
        }
    }
}
