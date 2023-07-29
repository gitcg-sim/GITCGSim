use std::sync::atomic::{AtomicBool, Ordering};

use rand::thread_rng;

#[cfg(not(feature = "no_parallel"))]
use {rayon::prelude::*, std::ops::Add};

use crate::{
    cons,
    minimax::{transposition_table::TTEntry, Windowable},
};
use crate::{game_tree_search::*, types::game_state::PlayerId};

use crate::linked_list;

use super::{
    transposition_table::{TTFlag, TT},
    EvalTrait, Game, SearchCounter,
};

/// Advance the game state while advancing the corresponding PV. Will discard the entire PV if the move does not match.
#[inline]
pub fn advance_with_pv<G: Game>(game: &mut G, pv: &PV<G>, input: G::Action) -> Result<PV<G>, G::Error> {
    game.advance(input)?;
    let pv1 = if Some(input) == pv.head() {
        pv.tail().unwrap()
    } else {
        linked_list![]
    };
    Ok(pv1)
}

#[allow(dead_code)]
const LAZY_SMP: bool = true;
const ASPIRATION_WINDOWS: bool = true;
const ITERATIVE_DEEPENING: bool = true;
const ITERATIVE_DEEPENING_STEP: u8 = 2;

/// Enable tactical search
const TACTICAL_SEARCH: bool = true;

/// Max. depth for tactical search
const TACTICAL_SEARCH_DEPTH: u8 = 8;

const TARGET_ROUND_DELTA: u8 = 2;

const STATIC_SEARCH_MAX_ITERS: u8 = 20;

/// Use null window search in the sequential portion.
const NULL_WINDOW: bool = false;

#[derive(Copy, Clone)]
struct SearchState<'a, 'b, G: Game> {
    pub tt: &'b TT<G::Eval, G::Action>,
    pub maximize_player: PlayerId,
    pub depth: u8,
    pub ab: (G::Eval, G::Eval),
    pub pv: &'a PV<G>,
    pub lazy_smp_index: Option<(u8, &'b LazySMPState<'b>)>,
    pub target_round: u8,
}

#[derive(Copy, Clone)]
enum DepthTransitionState {
    LazySMPRoot { depth: u8 },
    Full,
    Tactical,
}

struct SearchContext<'b, G: Game> {
    pub counter: SearchCounter,
    pub target_round: u8,
    pub lazy_smp_index: Option<(u8, &'b LazySMPState<'b>)>,
    pub tt: &'b TT<G::Eval, G::Action>,
}

impl<'b, G: Game> SearchContext<'b, G> {
    #[inline]
    pub fn lazy_smp_finished(&self) -> bool {
        let Some((_, lazy_smp)) = self.lazy_smp_index else { return false };
        lazy_smp.finished.load(Ordering::SeqCst)
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

fn tactical_search_rec<G: Game>(
    game: &G,
    maximize_player: PlayerId,
    ab: (G::Eval, G::Eval),
    depth: u8,
    pv: &PV<G>,
    ctx: &mut SearchContext<G>,
    dts: DepthTransitionState,
) -> (G::Eval, PV<G>) {
    if game.winner().is_some() || game.round_number() >= ctx.target_round || ctx.lazy_smp_finished() {
        ctx.counter.evals += 1;
        return (eval_position(game, maximize_player), linked_list![]);
    }

    if depth == 0 {
        return match dts {
            DepthTransitionState::LazySMPRoot { depth: _ } => {
                // tactical_search_rec(game, maximize_player, ab, depth, pv, ctx, DepthTransitionState::Full)
                todo!()
            }
            DepthTransitionState::Full => {
                let mut game = game.clone();
                game.convert_to_tactical_search();
                tactical_search_rec(
                    &game,
                    maximize_player,
                    ab,
                    depth,
                    pv,
                    ctx,
                    DepthTransitionState::Tactical,
                )
            }
            DepthTransitionState::Tactical => {
                ctx.counter.evals += 1;
                let (eval, n) = static_search(game, maximize_player, ctx.target_round, STATIC_SEARCH_MAX_ITERS);
                ctx.counter.states_visited += n;
                return (eval, linked_list![]);
            }
        };
    }

    let player = game.to_move().unwrap();

    let (mut alpha, beta) = ab;
    if player != maximize_player {
        let (e, pv1) = tactical_search_rec(game, maximize_player.opposite(), (-beta, -alpha), depth, pv, ctx, dts);
        return (-e, pv1);
    }

    let mut hit = false;
    let mut tt_depth = 0;
    if let Some(tt_res) = probe_tt(ctx.tt, game, depth, (alpha, beta), &mut hit, &mut tt_depth) {
        if tt_depth >= depth {
            return (tt_res.eval, tt_res.pv);
        }
    }

    if hit {
        ctx.counter.tt_hits += 1;
    }

    let mut write_back_tt = true;
    let alpha0 = alpha;
    let mut actions = game.actions();
    let mut pv = pv.clone();
    // internal iterative deepening
    if pv.is_empty() && depth >= 2 {
        let iid_depth = 1 + depth / 4;
        for current_depth in (1..iid_depth).step_by(2) {
            pv = tactical_search_rec::<G>(game, maximize_player, (alpha, beta), current_depth, &pv, ctx, dts).1;
        }
    }

    G::move_ordering(game, &pv, &mut actions);
    let mut best = G::Eval::MIN;
    for action in actions {
        if ctx.lazy_smp_finished() {
            write_back_tt = false;
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
        ctx.counter.states_visited += 1;
        let (eval, pv_rest) =
            tactical_search_rec(&game, maximize_player, (alpha, beta), depth - 1, &pv_inner, ctx, dts);

        if eval >= beta {
            ctx.counter.beta_prunes += 1;
            break;
        }

        if eval > best {
            pv = cons!(action, pv_rest);
            best = eval;
        }

        if eval > alpha {
            alpha = eval;
        }
    }

    if best < alpha {
        ctx.counter.all_nodes += 1;
    }

    if write_back_tt {
        let hash = game.zobrist_hash();
        let key = TT::<G::Eval, G::Action>::to_key(hash);
        if depth >= tt_depth {
            let entry = {
                let flag = tt_flag(alpha, alpha0, ab);
                TTEntry::new(flag, depth, alpha, pv.clone())
            };
            ctx.tt.table.pin().insert(key, entry);
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
    for current_depth in (1..=depth).step_by(2) {
        let (eval_next, pv_next) = tactical_search_rec(
            game,
            maximize_player,
            (G::Eval::MIN, G::Eval::MAX),
            current_depth,
            &pv,
            ctx,
            DepthTransitionState::Tactical,
        );
        eval = eval_next;
        if !pv_next.is_empty() {
            pv = pv_next;
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
            break
        };
        if game.round_number() >= target_round {
            break;
        }
        game.advance(action).expect("static_search: advance error");
        count += 1
    }

    (game.eval(maximize_player), count)
}

fn minimax<G: Game>(game: &G, ss: SearchState<G>) -> SearchResult<G> {
    let SearchState {
        maximize_player,
        mut depth,
        tt,
        ab,
        pv,
        lazy_smp_index,
        target_round,
    } = ss;

    let mut pv = pv.clone();

    let Some(to_move) = game.to_move() else {
        return handle_no_to_move(game, maximize_player)
    };

    if ss.maximize_player != to_move {
        return minimax(
            game,
            SearchState {
                maximize_player: maximize_player.opposite(),
                ab: (-ab.1, -ab.0),
                ..ss
            },
        )
        .negate();
    }

    {
        if game.round_number() >= target_round {
            depth = 0;
        }
    }

    let depth = depth; // No longer mutable
    let mut use_tt = true;
    let alpha0 = ab.0;
    let mut hit = false;
    let mut tt_depth = 0;
    let tt_res = if use_tt {
        probe_tt(tt, game, depth, ab, &mut hit, &mut tt_depth)
    } else {
        None
    };
    if let Some(return_value) = tt_res {
        if depth == 0 || !return_value.pv.is_empty() {
            return return_value;
        }
    }
    let ab = ab;

    if let Some((_, lazy_smp)) = lazy_smp_index {
        if lazy_smp.finished.load(Ordering::SeqCst) {
            return probe_tt(tt, game, depth, ab, &mut hit, &mut tt_depth)
                .unwrap_or_else(|| SearchResult::new(linked_list![], ab.0, SearchCounter::default()));
        }
    }

    if depth == 0 {
        let mut counter = SearchCounter::default();
        let eval = if TACTICAL_SEARCH {
            let mut game = game.clone();
            game.convert_to_tactical_search();
            let mut ctx = SearchContext {
                counter: Default::default(),
                target_round,
                lazy_smp_index,
                tt,
            };
            let res = tactical_search_iterative_deepening(
                &game,
                maximize_player,
                (G::Eval::MIN, G::Eval::MAX),
                TACTICAL_SEARCH_DEPTH,
                &mut ctx,
            );
            counter.add_in_place(&ctx.counter);
            res
        } else {
            counter.evals += 1;
            eval_position(game, maximize_player)
        };

        let hash = game.zobrist_hash();
        let key = TT::<G::Eval, G::Action>::to_key(hash);
        let entry = {
            let value = eval;
            let flag = tt_flag::<G::Eval>(value, alpha0, ab);
            TTEntry::new(flag, depth, value, linked_list![])
        };
        tt.table.pin().insert(key, entry);

        return SearchResult::new(linked_list![], eval, counter);
    }

    // Recursively perform the search
    let recurse = |game: &G, input: G::Action, ab, pv: &PV<G>| {
        let mut game = game.clone();
        let pv = advance_with_pv(&mut game, pv, input).unwrap();
        minimax(
            &game,
            SearchState {
                maximize_player,
                depth: depth - 1,
                ab,
                pv: &pv,
                tt,
                lazy_smp_index,
                target_round,
            },
        )
        .add_input_and_increment_counter(input)
    };

    let mut iid_counter = SearchCounter::default();
    // internal iterative deepening
    if pv.is_empty() && depth >= 2 {
        let iid_depth = 1 + depth / 4;
        for current_depth in (1..=iid_depth).step_by(2) {
            let game = game.clone();
            let res_iid = minimax(
                &game,
                SearchState {
                    maximize_player,
                    depth: current_depth,
                    ab,
                    pv: &pv,
                    tt,
                    lazy_smp_index,
                    target_round,
                },
            );
            iid_counter.add_in_place(&res_iid.counter);
            pv = res_iid.pv;
        }
    }

    let (res, alpha) = {
        // Sequential portion
        let (mut alpha, beta) = ab;
        let mut best = SearchResult::default();
        best.counter.add_in_place(&iid_counter);
        let mut actions = game.actions();
        let mut abort = None;
        let mut shuffle = false;
        if let Some((_, lazy_smp)) = lazy_smp_index {
            abort = Some(&lazy_smp.finished);
            shuffle = lazy_smp.top_depth == depth;
        }

        if shuffle {
            G::shuffle_actions(&mut actions, &mut thread_rng());
        } else {
            game.move_ordering(&pv, &mut actions);
        }

        let mut best_move = None;
        let mut do_pvs = true;
        for act in actions {
            if let Some(a) = abort {
                if a.load(Ordering::SeqCst) {
                    use_tt = false;
                    break;
                }
            }

            if let Some((_, lazy_smp)) = lazy_smp_index {
                if lazy_smp.finished.load(Ordering::SeqCst) {
                    use_tt = false;
                    best = SearchResult::default();
                    best.eval = G::Eval::MIN;
                    hit = false;
                    break;
                }
            }

            let res = if NULL_WINDOW && !do_pvs {
                let res_zws = recurse(game, act, alpha.null_window(), &pv);
                if alpha < res_zws.eval && res_zws.eval < beta {
                    best.counter.zws_fails += 1;
                    // res0 thrown away, need to manually add counter
                    best.counter.add_in_place(&res_zws.counter);
                    recurse(game, act, (alpha, beta), &pv)
                } else {
                    res_zws
                }
            } else {
                recurse(game, act, (alpha, beta), &pv)
            };

            if res.eval > best.eval {
                best_move = Some(act);
            }
            best.update(&res);

            if res.eval >= beta {
                best.counter.beta_prunes += 1;
                break;
            }

            if res.eval > alpha {
                alpha = best.eval;
                do_pvs = false;
                pv = cons!(act, res.pv);
            }
        }

        if best.eval < alpha {
            best.counter.all_nodes += 1;
        }

        if best.pv.is_empty() {
            if let Some(act) = best_move {
                best.pv = linked_list![act];
            }
        }

        (best, alpha)
    };

    if use_tt {
        let hash = game.zobrist_hash();
        let key = TT::<G::Eval, G::Action>::to_key(hash);
        if depth >= tt_depth {
            let entry = {
                let flag = tt_flag(alpha, alpha0, ab);
                TTEntry::new(flag, depth, alpha, res.pv.clone())
            };
            tt.table.pin().insert(key, entry);
        }
    }

    let mut res = res;
    if hit {
        res.counter.tt_hits += 1;
    }
    res
}

#[inline]
fn tt_flag<E: EvalTrait>(value: E, alpha0: E, ab: (E, E)) -> TTFlag {
    if value <= alpha0 {
        TTFlag::Upper
    } else if value >= ab.1 {
        TTFlag::Lower
    } else {
        TTFlag::Exact
    }
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
        ..
    } = *entry;
    if depth_from_tt < depth {
        return None;
    }

    let early_exit = {
        *hit = true;
        let pv = entry.pv.clone();
        move |value: G::Eval| {
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

fn handle_no_to_move<G: Game>(game: &G, maximize_player: PlayerId) -> SearchResult<G> {
    let Some(winner) = game.winner() else {
        panic!("handle_no_move: Error: {maximize_player} {game:?}")
    };
    if winner == maximize_player {
        SearchResult::new(linked_list![], G::Eval::MAX, SearchCounter::EVAL)
    } else {
        SearchResult::new(linked_list![], G::Eval::MIN, SearchCounter::EVAL)
    }
}

struct LazySMPState<'a> {
    pub finished: &'a AtomicBool,
    pub top_depth: u8,
}

fn minimax_lazy_smp<G: Game>(
    #[allow(unused_variables)] parallel: bool,
    game: &G,
    ss: SearchState<G>,
) -> Option<SearchResult<G>> {
    #[cfg(feature = "no_parallel")]
    let ret = minimax(game, ss);

    #[cfg(not(feature = "no_parallel"))]
    let ret = if parallel && LAZY_SMP {
        let threads = 24;
        let depth = ss.depth;
        let finished = &AtomicBool::default();

        let (total_counter, mut res) = rayon::join(
            {
                let game_1 = game.clone();
                move || {
                    (1..threads)
                        .into_par_iter()
                        .map(move |i| {
                            if i >= 8 {
                                std::thread::yield_now();
                            }
                            if finished.load(Ordering::SeqCst) {
                                return SearchCounter::default();
                            }
                            let depth = depth
                                + match i * (if threads >= 12 { 1 } else { 2 }) {
                                    0 => 4,
                                    1 => 3,
                                    2 => 2,
                                    3 => 2,
                                    4 => 1,
                                    5 => 1,
                                    6 => 1,
                                    7 => 1,
                                    _ => 0,
                                };
                            let lazy_smp = LazySMPState {
                                finished,
                                top_depth: depth,
                            };
                            minimax(
                                &game_1,
                                SearchState {
                                    lazy_smp_index: Some((i, &lazy_smp)),
                                    depth,
                                    ..ss
                                },
                            )
                            .counter
                        })
                        .reduce(SearchCounter::default, SearchCounter::add)
                }
            },
            move || {
                let tt = ss.tt;
                let ss = SearchState {
                    lazy_smp_index: None,
                    ..ss
                };
                let mut res = minimax(game, ss.clone());
                for _ in 0..2 {
                    if res.pv.head().is_some() {
                        break;
                    }
                    tt.pin().tt.clear();
                    let ss = SearchState {
                        lazy_smp_index: None,
                        ..ss.clone()
                    };
                    res = minimax(game, ss);
                }
                finished.store(true, Ordering::SeqCst);
                res
            },
        );

        res.counter.add_in_place(&total_counter);
        res
    } else {
        minimax(game, ss)
    };
    if ret.pv.head().is_some() && game.clone().advance(ret.pv.head().unwrap()).is_ok() {
        Some(ret)
    } else {
        None
    }
}

fn minimax_iterative_deepening_aspiration_windows<G: Game>(
    game: &G,
    tt: &TT<G::Eval, G::Action>,
    maximize_player: PlayerId,
    depth: u8,
    parallel: bool,
) -> SearchResult<G> {
    let default_window: (G::Eval, G::Eval) = (G::Eval::MIN, G::Eval::MAX);
    let ss0 = SearchState {
        maximize_player,
        depth,
        ab: default_window,
        pv: &linked_list![],
        tt,
        lazy_smp_index: None,
        target_round: game.round_number() + TARGET_ROUND_DELTA,
    };
    const STEP: u8 = ITERATIVE_DEEPENING_STEP;
    if ITERATIVE_DEEPENING {
        let depth0 = if depth > STEP { STEP } else { depth };
        let search = |current_depth, window, pv| {
            let Some(SearchResult { pv, eval, counter }) = minimax_lazy_smp(
                parallel,
                game,
                SearchState {
                    tt,
                    depth: current_depth,
                    pv: &pv,
                    ab: window,
                    ..ss0
                },
            ) else {
                return None;
            };
            Some((pv, eval, counter))
        };

        let (mut pv, mut eval, mut counter) = search(1, (G::Eval::MIN, G::Eval::MAX), linked_list![]).unwrap();

        for current_depth in (depth0..=depth).step_by(STEP as usize) {
            let mut found = false;
            let window = if ASPIRATION_WINDOWS {
                let mut window = eval.aspiration_window();
                for step in 0..=0_u8 {
                    let Some((pv1, value, counter1)) = search(current_depth, window, pv.clone()) else {
                        break
                    };
                    counter.add_in_place(&counter1);
                    counter.aw_iters += 1;
                    if window.0 < value && value < window.1 {
                        found = true;
                        pv = pv1.clone();
                        eval = value;
                        break;
                    } else {
                        window = if value < window.0 {
                            counter.aw_fail_lows += 1;
                            (value.minus_unit(step), window.1)
                        } else {
                            counter.aw_fail_highs += 1;
                            (window.0, value.plus_unit(step))
                        };
                    }
                }
                window
            } else {
                (G::Eval::MIN, G::Eval::MAX)
            };

            // let pv_vec: Vec<_> = pv.clone().into_iter().collect();
            // println!("{current_depth:2}: Eval={eval:?}, PV={pv_vec:?}");
            if !found {
                let Some((pv1, eval1, counter1)) = search(current_depth, window, pv.clone()) else {
                    break
                };
                eval = eval1;
                pv = pv1.clone();
                counter.add_in_place(&counter1);
            }
        }
        SearchResult::new(pv, eval, counter)
    } else {
        minimax(game, ss0)
    }
}

pub struct MinimaxConfig {
    pub depth: u8,
    pub parallel: bool,
    pub tt_size: usize,
}

impl MinimaxConfig {
    pub fn new(depth: u8, parallel: bool, tt_size: Option<usize>) -> Self {
        Self {
            depth,
            parallel,
            tt_size: tt_size.unwrap_or(super::transposition_table::DEFAULT_SIZE),
        }
    }
}

pub struct MinimaxSearch<G: Game> {
    pub tt: TT<G::Eval, G::Action>,
    pub config: MinimaxConfig,
}

impl<G: Game> MinimaxSearch<G> {
    pub fn new(config: MinimaxConfig) -> Self {
        let tt = TT::<G::Eval, G::Action>::new(config.tt_size);
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
        )
    }
}

impl<G: Game> Drop for MinimaxSearch<G> {
    fn drop(&mut self) {
        self.tt.pin().tt.clear();
    }
}
