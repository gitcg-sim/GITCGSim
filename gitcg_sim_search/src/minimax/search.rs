use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Instant,
};
#[cfg(not(feature = "no_parallel"))]
use {rayon::prelude::*, std::ops::Add};

use gitcg_sim::{cons, linked_list, prelude::*, rand::thread_rng};

use crate::*;

use crate::minimax::transposition_table::{TTEntry, TTFlag, TT};

fn widen_aspiration_window<G: Game>(
    value: G::Eval,
    step: u8,
    window: (G::Eval, G::Eval),
    #[allow(unused_variables)] counter: &mut SearchCounter,
) -> (G::Eval, G::Eval) {
    let (alpha, beta) = window;
    if value <= alpha {
        #[cfg(feature = "detailed_search_stats")]
        {
            counter.aw_fail_lows += 1;
        }
        (value.minus_unit(step), beta)
    } else {
        #[cfg(feature = "detailed_search_stats")]
        {
            counter.aw_fail_highs += 1;
        }
        (alpha, value.plus_unit(step))
    }
}

#[inline]
fn is_within_aspiration_window<T: PartialOrd + Ord>(value: T, window: (T, T)) -> bool {
    let (alpha, beta) = window;
    alpha < value && value < beta
}

#[allow(dead_code)]
const LAZY_SMP: bool = true;
const ASPIRATION_WINDOWS: bool = true;
const ITERATIVE_DEEPENING_STEP: u8 = 2;

/// Max. depth for tactical search
pub const TACTICAL_SEARCH_DEPTH: u8 = 10;

pub const TARGET_ROUND_DELTA: u8 = 2;

pub const STATIC_SEARCH_MAX_ITERS: u8 = 20;

#[derive(Copy, Clone)]
enum DepthTransitionState {
    Full,
    Tactical,
}

type ThreadId = u8;

struct SearchContext<'b, G: Game> {
    pub config: MinimaxConfig,
    pub counter: SearchCounter,
    pub start_time: Instant,
    pub target_round: u8,
    pub lazy_smp_index: Option<(ThreadId, &'b LazySMPState<'b>)>,
    pub tt: &'b TT<G::Eval, G::Action>,
}

impl<'b, G: Game> SearchContext<'b, G> {
    #[inline]
    pub fn lazy_smp_finished(&self) -> bool {
        let Some((_, lazy_smp)) = self.lazy_smp_index else {
            return false;
        };
        lazy_smp.finished.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn should_terminate(&self) -> bool {
        let Some(limits) = &self.config.limits else {
            return false;
        };

        let positions_searched = if let Some((_, lazy_smp)) = self.lazy_smp_index {
            lazy_smp.positions_searched.load(Ordering::SeqCst)
        } else {
            self.counter.states_visited
        };
        limits.should_terminate(self.start_time, positions_searched)
    }

    #[inline]
    pub fn add_states_visited(&mut self, value: u64) {
        self.counter.states_visited += value;
        if let Some((_, lazy_smp)) = self.lazy_smp_index {
            lazy_smp.positions_searched.fetch_add(value, Ordering::SeqCst);
        };
    }

    #[inline]
    pub fn should_shuffle_actions(&self, depth: u8, dts: DepthTransitionState) -> bool {
        let DepthTransitionState::Full = dts else { return false };
        let Some((_, lazy_smp)) = self.lazy_smp_index else {
            return false;
        };
        lazy_smp.top_depth == depth
    }
}

#[inline]
fn eval_position<G: Game>(game: &G, maximize_player: PlayerId) -> G::Eval {
    if G::PREPARE_FOR_EVAL {
        let mut game = game.clone();
        game.prepare_for_eval();
        game.eval(maximize_player)
    } else {
        game.eval(maximize_player)
    }
}

fn minimax<G: Game>(
    game: &G,
    maximize_player: PlayerId,
    ab: (G::Eval, G::Eval),
    depth: u8,
    pv: &PV<G>,
    ctx: &mut SearchContext<G>,
    dts: DepthTransitionState,
) -> (G::Eval, PV<G>) {
    if game.winner().is_some()
        || game.round_number() >= ctx.target_round
        || ctx.should_terminate()
        || ctx.lazy_smp_finished()
    {
        ctx.counter.evals += 1;
        return (eval_position(game, maximize_player), linked_list![]);
    }

    if depth == 0 {
        return match dts {
            DepthTransitionState::Full => {
                let mut game = game.clone();
                game.convert_to_tactical_search();
                (
                    tactical_search_iterative_deepening(
                        &game,
                        maximize_player,
                        (G::Eval::MIN, G::Eval::MAX),
                        ctx.config.tactical_depth,
                        ctx,
                    ),
                    linked_list![],
                )
            }
            DepthTransitionState::Tactical => {
                ctx.counter.evals += 1;
                let (eval, n) = static_search(
                    game,
                    maximize_player,
                    ctx.target_round,
                    ctx.config.static_search_max_iters,
                );
                ctx.add_states_visited(n);
                (eval, linked_list![])
            }
        };
    }

    let player = game.to_move().unwrap();

    let (mut alpha, beta) = ab;
    if player != maximize_player {
        let (e, pv1) = minimax(game, maximize_player.opposite(), (-beta, -alpha), depth, pv, ctx, dts);
        return (-e, pv1);
    }

    let mut hit = false;
    let mut tt_depth = 0;
    if let Some(tt_res) = probe_tt(ctx.tt, game, depth, (alpha, beta), &mut hit, &mut tt_depth) {
        if tt_depth >= depth && !tt_res.pv.is_empty() {
            ctx.counter.tt_hits += 1;
            return (tt_res.eval, tt_res.pv);
        }
    }

    if hit {
        ctx.counter.tt_hits += 1;
    }

    let mut write_back_tt = true;
    let mut actions = game.actions();
    let mut pv = pv.clone();
    // internal iterative deepening
    if pv.is_empty() && depth > 2 {
        let iid_depth = 1 + depth / 4;
        for current_depth in (1..iid_depth).step_by(2) {
            pv = minimax::<G>(game, maximize_player, (alpha, beta), current_depth, &pv, ctx, dts).1;
        }
    }

    if ctx.should_shuffle_actions(depth, dts) {
        G::shuffle_actions(&mut actions, &mut thread_rng());
    } else {
        G::move_ordering(game, &pv, &mut actions);
    }
    let mut best = G::Eval::MIN;
    let mut best_move = None;
    let mut flag = TTFlag::Upper;
    for action in actions {
        if ctx.should_terminate() || ctx.lazy_smp_finished() {
            write_back_tt = false;
            break;
        }

        let pv_inner = match pv.decons() {
            None => linked_list![],
            Some((act1, rest)) => {
                if act1 == action {
                    rest
                } else {
                    linked_list![]
                }
            }
        };
        let mut game = game.clone();
        game.advance(action).unwrap();
        ctx.add_states_visited(1);
        let new_depth = depth - 1 + game.depth_extension(action);
        let (eval, pv_rest) = minimax(&game, maximize_player, (alpha, beta), new_depth, &pv_inner, ctx, dts);

        if eval >= beta {
            #[cfg(feature = "detailed_search_stats")]
            {
                ctx.counter.beta_prunes += 1;
            }
            flag = TTFlag::Lower;
            break;
        }

        if eval > best {
            pv = cons!(action, pv_rest);
            best_move = Some(action);
            best = eval;
        }

        if eval > alpha {
            alpha = eval;
            flag = TTFlag::Exact;
        }
    }

    if pv.is_empty() {
        if let Some(best_move) = best_move {
            pv = linked_list![best_move];
        }
    }

    #[cfg(feature = "detailed_search_stats")]
    if best <= alpha {
        ctx.counter.all_nodes += 1;
    }

    if write_back_tt {
        let hash = game.zobrist_hash();
        let key = TT::<G::Eval, G::Action>::to_key(hash);
        if depth >= tt_depth {
            let entry = TTEntry::new(flag, depth, alpha, pv.clone());
            ctx.tt.pin().insert(key, entry);
        }
    }

    (alpha, pv)
}

fn tactical_search_iterative_deepening<G: Game>(
    game: &G,
    maximize_player: PlayerId,
    ab: (G::Eval, G::Eval),
    depth: u8,
    ctx: &mut SearchContext<G>,
) -> G::Eval {
    let mut eval = ab.0;
    let mut pv = linked_list![];
    let depth0 = if depth > ITERATIVE_DEEPENING_STEP {
        ITERATIVE_DEEPENING_STEP
    } else {
        depth
    };
    'iterative_deepening_loop: for current_depth in (depth0..=depth).step_by(ITERATIVE_DEEPENING_STEP as usize) {
        let mut window = ab;
        let mut found = false;
        let search = {
            let pv = pv.clone();
            move |window: (G::Eval, G::Eval), ctx: &mut SearchContext<G>| {
                minimax(
                    game,
                    maximize_player,
                    window,
                    current_depth,
                    &pv,
                    ctx,
                    DepthTransitionState::Tactical,
                )
            }
        };

        if ASPIRATION_WINDOWS {
            'aspiration_loop: for step in 1..=2u8 {
                if ctx.lazy_smp_finished() || ctx.should_terminate() {
                    break 'iterative_deepening_loop;
                }

                let (value, pv_next) = search(window, ctx);
                if is_within_aspiration_window(value, window) {
                    found = true;
                    pv = pv_next.clone();
                    eval = value;
                    break 'aspiration_loop;
                } else {
                    window = widen_aspiration_window::<G>(value, step, window, &mut ctx.counter);
                }
            }
        } else if ctx.lazy_smp_finished() || ctx.should_terminate() {
            break 'iterative_deepening_loop;
        }

        if found {
            continue;
        }

        let (value, pv_next) = search(window, ctx);
        eval = value;
        if !pv_next.is_empty() {
            pv = pv_next.clone();
        }
    }
    eval
}

fn static_search<G: Game>(game: &G, maximize_player: PlayerId, target_round: u8, depth: u8) -> (G::Eval, u64) {
    let mut game = game.clone();
    let mut count = 0u64;
    for _ in 0..depth {
        if game.winner().is_some() {
            break;
        }
        let Some(action) = game.static_search_action(maximize_player) else {
            break;
        };
        if game.round_number() >= target_round {
            break;
        }
        game.advance(action).expect("static_search: advance error");
        count += 1
    }

    (game.eval(maximize_player), count)
}

fn probe_tt<G: Game>(
    tt: &TT<G::Eval, G::Action>,
    game: &G,
    depth: u8,
    ab: (G::Eval, G::Eval),
    hit: &mut bool,
    tt_depth: &mut u8,
) -> Option<SearchResult<G>> {
    let hash = game.zobrist_hash();
    let key = TT::<G::Eval, G::Action>::to_key(hash);
    let tt_ref = tt.pin();
    let Some(entry) = tt_ref.get(&key) else { return None };
    let TTEntry {
        flag,
        value,
        depth: depth_from_tt,
        pv,
        ..
    } = {
        #[cfg(feature = "old_tt")]
        {
            *entry
        }
        #[cfg(not(feature = "old_tt"))]
        entry
    };
    if depth_from_tt < depth {
        return None;
    }

    let early_exit = {
        #[cfg(feature = "old_tt")]
        let pv = pv.clone();
        #[cfg(not(feature = "old_tt"))]
        let pv = pv;

        move |value: G::Eval| {
            *hit = true;
            *tt_depth = depth;
            Some(SearchResult::new(pv, value, SearchCounter::HIT))
        }
    };

    match flag {
        TTFlag::Exact => {
            return early_exit(value);
        }
        TTFlag::Upper => {
            if value <= ab.0 {
                return early_exit(ab.0);
            }
        }
        TTFlag::Lower => {
            if value >= ab.1 {
                return early_exit(ab.1);
            }
        }
    }

    None
}

struct LazySMPState<'a> {
    pub finished: &'a AtomicBool,
    pub positions_searched: &'a AtomicU64,
    pub top_depth: u8,
}

fn minimax_lazy_smp<G: Game>(
    #[allow(unused_variables)] parallel: bool,
    game: &G,
    maximize_player: PlayerId,
    ab: (G::Eval, G::Eval),
    depth: u8,
    pv: &PV<G>,
    ctx: &mut SearchContext<G>,
) -> Option<(G::Eval, PV<G>)> {
    #[cfg(feature = "no_parallel")]
    let (eval, pv) = minimax(game, maximize_player, ab, depth, pv, ctx, DepthTransitionState::Full);

    #[cfg(not(feature = "no_parallel"))]
    let (eval, pv) = if parallel && LAZY_SMP {
        let threads: ThreadId = 24;
        let finished = &AtomicBool::default();
        let positions_searched = &AtomicU64::new(ctx.counter.states_visited);
        let SearchContext {
            target_round,
            tt,
            config,
            start_time,
            lazy_smp_index: _,
            counter: _,
        } = *ctx;

        let (total_counter, (eval, pv, search_counter)) = rayon::join(
            {
                let game_1 = game.clone();
                move || {
                    (1..threads)
                        .into_par_iter()
                        .map(move |thread_id| {
                            if thread_id >= 8 {
                                std::thread::yield_now();
                            }
                            if finished.load(Ordering::SeqCst) {
                                return SearchCounter::default();
                            }
                            // let depth = depth + std::cmp::min(3, thread_id.trailing_ones() as u8);
                            let depth = depth + depth % 2;
                            let lazy_smp = LazySMPState {
                                finished,
                                top_depth: depth,
                                positions_searched,
                            };
                            let mut ctx = SearchContext {
                                counter: SearchCounter::default(),
                                target_round,
                                config,
                                start_time,
                                lazy_smp_index: Some((thread_id, &lazy_smp)),
                                tt,
                            };
                            minimax(
                                &game_1,
                                maximize_player,
                                ab,
                                depth,
                                pv,
                                &mut ctx,
                                DepthTransitionState::Full,
                            );
                            ctx.counter
                        })
                        .reduce(SearchCounter::default, SearchCounter::add)
                }
            },
            move || {
                let mut ctx = SearchContext {
                    counter: SearchCounter::default(),
                    target_round,
                    config,
                    start_time,
                    lazy_smp_index: None,
                    tt,
                };
                let (eval, pv) = minimax(
                    game,
                    maximize_player,
                    ab,
                    depth,
                    pv,
                    &mut ctx,
                    DepthTransitionState::Full,
                );
                finished.store(true, Ordering::SeqCst);
                (eval, pv, ctx.counter)
            },
        );

        ctx.counter.add_in_place(&total_counter);
        ctx.counter.add_in_place(&search_counter);
        (eval, pv)
    } else {
        let (eval, pv) = minimax(game, maximize_player, ab, depth, pv, ctx, DepthTransitionState::Full);
        (eval, pv)
    };
    if pv.head().is_some() && game.clone().advance(pv.head().unwrap()).is_ok() {
        Some((eval, pv))
    } else {
        None
    }
}

#[inline]
fn minimax_iterative_deepening_aspiration_windows<G: Game>(
    game: &G,
    tt: &TT<G::Eval, G::Action>,
    maximize_player: PlayerId,
    depth: u8,
    parallel: bool,
    config: MinimaxConfig,
) -> SearchResult<G> {
    let full_window: (G::Eval, G::Eval) = (G::Eval::MIN, G::Eval::MAX);
    let mut ctx0 = SearchContext {
        counter: SearchCounter::default(),
        tt,
        lazy_smp_index: None,
        config,
        start_time: Instant::now(),
        target_round: game.round_number() + config.target_round_delta,
    };
    const STEP: u8 = ITERATIVE_DEEPENING_STEP;

    let depth0 = if depth > STEP { STEP } else { depth };
    macro_rules! search {
        ($current_depth: expr, $ab: expr, $pv: expr $(,)?) => {
            minimax_lazy_smp(
                parallel,
                game,
                maximize_player,
                $ab,
                $current_depth,
                &$pv,
                &mut ctx0,
            )
        };
    }

    let (mut eval, mut pv) = search!(std::cmp::max(1, depth0.saturating_sub(1)), full_window, linked_list![])
        .unwrap_or((G::Eval::MIN, linked_list![]));

    'iterative_deepening_loop: for current_depth in (depth0..=depth).step_by(STEP as usize) {
        let mut found = false;
        let mut last_visited = 0;
        if ASPIRATION_WINDOWS {
            let mut window = eval.aspiration_window();
            // For debugging
            'aspiration_loop: for step in 1..=3_u8 {
                if ctx0.should_terminate() {
                    break 'iterative_deepening_loop;
                }

                let Some((value, pv1)) = search!(current_depth, window, pv.clone()) else {
                    break 'aspiration_loop;
                };

                let next_visited = ctx0.counter.states_visited;

                if config.debug {
                    println!(
                        "  --> AW: {window:?} | value={value:?}, within_range={}, states_visited={}",
                        is_within_aspiration_window(value, window),
                        next_visited - last_visited
                    );
                    last_visited = next_visited;
                }

                #[cfg(feature = "detailed_search_stats")]
                {
                    ctx0.counter.aw_iters += 1;
                }

                if is_within_aspiration_window(value, window) {
                    found = true;
                    ctx0.counter.last_depth = current_depth;
                    pv = pv1.clone();
                    eval = value;
                    break 'aspiration_loop;
                } else {
                    window = widen_aspiration_window::<G>(value, step, window, &mut ctx0.counter);
                }
            }
        } else if ctx0.should_terminate() {
            break 'iterative_deepening_loop;
        }

        if !found {
            let Some((eval1, pv1)) = search!(current_depth, full_window, pv.clone()) else {
                break 'iterative_deepening_loop;
            };
            let next_visited = ctx0.counter.states_visited;
            if config.debug {
                println!("  --> AW: full search, states_visited={}", next_visited - last_visited);
            }
            ctx0.counter.last_depth = current_depth;
            eval = eval1;
            pv = pv1.clone();
        }
        if config.debug {
            let pv_vec = pv.into_iter().copied().collect::<Vec<_>>();
            println!(" - Depth {current_depth:2}: Eval={eval:?}, PV={pv_vec:?}");
        }
    }
    SearchResult::new(pv, eval, ctx0.counter)
}

#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MinimaxConfig {
    pub depth: u8,
    pub tactical_depth: u8,
    pub static_search_max_iters: u8,
    pub target_round_delta: u8,
    pub parallel: bool,
    pub tt_size_mb: u32,
    pub limits: Option<SearchLimits>,
    pub debug: bool,
}

pub struct MinimaxSearch<G: Game> {
    pub tt: TT<G::Eval, G::Action>,
    pub config: MinimaxConfig,
}

impl<G: Game> MinimaxSearch<G> {
    pub fn new(config: MinimaxConfig) -> Self {
        let tt = TT::<G::Eval, G::Action>::new(config.tt_size_mb);
        Self { tt, config }
    }
}

impl<G: Game> GameTreeSearch<G> for MinimaxSearch<G> {
    fn search(&mut self, position: &G, maximize_player: PlayerId) -> SearchResult<G> {
        minimax_iterative_deepening_aspiration_windows(
            position,
            &self.tt,
            maximize_player,
            self.config.depth,
            self.config.parallel,
            self.config,
        )
    }
}

impl<G: Game> Drop for MinimaxSearch<G> {
    fn drop(&mut self) {
        self.tt.pin().tt.clear();
    }
}
