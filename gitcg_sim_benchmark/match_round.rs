use gitcg_sim::rand::prelude::*;
use gitcg_sim_search::{GameTreeSearch, SearchCounter, SearchResult};
use instant::Duration;
use rayon::prelude::*;
use std::{
    sync::atomic::{AtomicI32, Ordering},
    time::Instant,
};

use gitcg_sim::prelude::*;

pub fn match_round<S: NondetState, T: GameTreeSearch<GameStateWrapper<S>>>(
    initial_state: GameStateWrapper<S>,
    searches: &mut ByPlayer<T>,
    steps: u32,
) -> (Option<PlayerId>, Duration, SearchCounter) {
    let mut game = initial_state;
    let t0 = Instant::now();
    let mut total_counter = SearchCounter::default();
    for _ in 0..steps {
        if game.winner().is_some() {
            break;
        }

        let p = game.to_move().unwrap();
        let search = &mut searches[p];
        let SearchResult {
            pv,
            eval: _,
            counter: c,
        } = search.search_hidden(&game, p);
        total_counter.add_in_place(&c);
        if pv.is_empty() {
            println!("perform_match: PV is empty.");
            break;
        }
        let input = pv.head().unwrap();
        if let Err(e) = game.advance(input) {
            println!("----------");
            println!("Error: {e:?}");
            println!("Input: {input:?}");
            println!("Game state: {game:?}");
            println!("----------");
            println!();
            break;
        };
    }

    (game.winner(), t0.elapsed(), total_counter)
}

pub fn trace_search<S: NondetState, T: GameTreeSearch<GameStateWrapper<S>>>(
    game: &GameStateWrapper<S>,
    searches: &mut ByPlayer<T>,
    steps: u32,
) -> (u128, SearchCounter) {
    let mut game = game.clone();
    let mut total_counter = SearchCounter::default();
    let mut total_time: u128 = 0;
    for i in 0..steps {
        let t1 = Instant::now();
        if let Some(winner) = game.winner() {
            println!("Winner: {winner:?}");
            break;
        }
        let p = game.to_move().unwrap();
        let search = searches.get_mut(p);
        let SearchResult {
            pv,
            eval: v,
            counter: c,
        } = search.search_hidden(&game, p);
        total_counter.add_in_place(&c);
        let input = pv.head().expect("PV is empty");
        let dt_ns = t1.elapsed().as_nanos();
        total_time += dt_ns;
        match game.advance(input) {
            Err(e) => {
                println!("DispatchError (to_move_player={:?}, input={input:?}):", game.to_move());
                println!("{game:?}");
                println!();
                println!("pv={pv:?} v={v:?}");
                println!("available_actions:");
                game.actions().into_iter().for_each(|x| println!(" - {x:?}"));
                panic!("trace_search: failed: {e:?} {input:?}");
            }
            _ => {
                println!("--> {i:2} {input:?} | {v:?} | {} | {c:?}", c.summary(dt_ns));
            }
        }
    }
    (total_time, total_counter)
}

#[derive(Default)]
pub struct IterateMatchOpts {
    pub rounds: u32,
    pub steps: u32,
    pub random_seed: u64,
}

pub fn iterate_match<
    S: NondetState,
    T: GameTreeSearch<GameStateWrapper<S>>,
    M: Send + Sync + Fn() -> ByPlayer<T>,
    G: Send + Sync + Fn(SmallRng) -> GameStateWrapper<S>,
>(
    make_search: &M,
    get_game: &G,
    opts: IterateMatchOpts,
) -> (i32, f32, SearchCounter) {
    let IterateMatchOpts {
        rounds,
        random_seed,
        steps,
    } = opts;
    let matches_started = AtomicI32::default();
    let (score, total_counter) = (0..rounds)
        .into_par_iter()
        .map(|_| {
            let i = matches_started.fetch_add(1, Ordering::SeqCst);
            let flip = i % 2 == 0;
            let mut search = make_search();
            if flip {
                std::mem::swap(&mut search.0, &mut search.1);
            }
            let rng = SmallRng::seed_from_u64(random_seed.wrapping_add(2).overflowing_mul(i as u64).0);
            let game = get_game(rng);

            println!("+ Round {:3}", i + 1);
            let (winner, dt, c) = match_round(game, &mut search, steps);
            let (winner_str, d_score) = get_winner_value(winner, flip);
            println!(
                "- Round {:3} ... {winner_str} dt={:6.2}ms, states_visited={:8}",
                i + 1,
                dt.as_millis(),
                c.states_visited
            );
            (d_score, c)
        })
        .reduce(
            || (Default::default(), Default::default()),
            |(s, mut c), (s1, c1)| {
                c.add_in_place(&c1);
                (s + s1, c)
            },
        );

    (score, (score as f32) / ((2 * rounds) as f32), total_counter)
}

pub fn get_winner_value(winner: Option<PlayerId>, flip: bool) -> (&'static str, i32) {
    match winner {
        Some(PlayerId::PlayerFirst) => {
            if flip {
                ("0-1", 0)
            } else {
                ("1-0", 2)
            }
        }
        Some(PlayerId::PlayerSecond) => {
            if flip {
                ("1-0", 2)
            } else {
                ("0-1", 0)
            }
        }
        None => ("1/2", 1),
    }
}
