use gitcg_sim_search::{GameTreeSearch, SearchCounter, SearchResult};
use instant::Duration;
use lazy_static::__Deref;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Instant;
use structopt::StructOpt;

use gitcg_sim::{
    prelude::*,
    rand::{rngs::SmallRng, SeedableRng},
};
use gitcg_sim_cli_utils::cli_args::{SearchAlgorithm, SearchConfig, SearchOpts};
use gitcg_sim_search::mcts::CpuctConfig;

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Genius Invokation TCG simulator")]
pub enum BenchmarkOpts {
    #[structopt(help = "Run a game in parallel execution then in sequential execution, and compare speedups.")]
    Speedup {
        #[structopt(flatten)]
        search: SearchOpts,
    },
    #[structopt(help = "Run a game in parallel or sequential execution.")]
    Benchmark {
        #[structopt(long = "--parallel", short = "-P")]
        parallel: bool,
        #[structopt(flatten)]
        search: SearchOpts,
    },
    #[structopt(help = "Evaluate the first move of a position.")]
    Evaluate {
        #[structopt(long = "--parallel", short = "-P")]
        parallel: bool,
        #[structopt(flatten)]
        search: SearchOpts,
    },
    #[structopt(help = "Measure win rate against a standarized opponent.")]
    Match {
        #[structopt(long = "--parallel", short = "-P")]
        parallel: bool,

        #[structopt(long)]
        steps: Option<u32>,

        #[structopt(long)]
        rounds: Option<u32>,

        #[structopt(flatten)]
        search: SearchOpts,

        #[structopt(long)]
        standard_algorithm: Option<SearchAlgorithm>,

        #[structopt(long)]
        standard_time_limit_ms: Option<u128>,
    },
}

impl BenchmarkOpts {
    fn deck(&self) -> &SearchOpts {
        match self {
            BenchmarkOpts::Speedup { search: deck, .. } => deck,
            BenchmarkOpts::Benchmark { search: deck, .. } => deck,
            BenchmarkOpts::Evaluate { search: deck, .. } => deck,
            BenchmarkOpts::Match { search: deck, .. } => deck,
        }
    }
}

fn trace_search<S: NondetState, T: GameTreeSearch<GameStateWrapper<S>>>(
    game: &GameStateWrapper<S>,
    steps: u32,
    searches: &RefCell<ByPlayer<T>>,
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
        let mut searches = searches.borrow_mut();
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

fn match_round<S: NondetState, T: GameTreeSearch<GameStateWrapper<S>>>(
    mut game: GameStateWrapper<S>,
    search: &mut ByPlayer<T>,
    steps: u32,
) -> (Option<PlayerId>, Duration, SearchCounter) {
    let t0 = Instant::now();
    let mut total_counter = SearchCounter::default();
    for _ in 0..steps {
        if game.winner().is_some() {
            break;
        }

        let p = game.to_move().unwrap();
        let search = &mut search[p];
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

fn standard_search_opts(algorithm: Option<SearchAlgorithm>, standard_time_limit_ms: Option<u128>) -> SearchConfig {
    SearchConfig {
        algorithm,
        mcts_cpuct_init: Some(CpuctConfig::STANDARD.init),
        mcts_cpuct_base: Some(CpuctConfig::STANDARD.base),
        mcts_cpuct_factor: Some(CpuctConfig::STANDARD.factor),
        mcts_random_playout_iters: Some(1),
        mcts_random_playout_max_steps: Some(20),
        mcts_random_playout_bias: Some(50.0),
        mcts_policy_bias: Some(1.0),
        mcts_use_policy_network: true,
        policy_based_bias: Some(1.0),
        time_limit_ms: standard_time_limit_ms.or(Some(300)),
        tt_size_mb: Some(32),
        search_depth: Some(4),
        tactical_depth: Some(6),
        ..Default::default()
    }
}

fn main() -> Result<(), std::io::Error> {
    let opts = BenchmarkOpts::from_args();
    let steps: u32 = opts.deck().steps.unwrap_or(200);
    let (bf, benchmark_half) = {
        let deck_opts: &SearchOpts = opts.deck();
        let search_opts = &deck_opts.search;
        let depth: u8 = search_opts.search_depth.unwrap_or(8);
        let bf = move |n: f64| n.powf(1_f64 / (depth as f64));
        let game = Rc::new(deck_opts.get_standard_game(None)?);
        let benchmark = move |parallel: bool, steps: u32| {
            let f = || deck_opts.make_search(parallel, deck_opts.get_limits());
            let searches = RefCell::new(ByPlayer::new(f(), f()));
            let game = game.deref();
            let (dt_ns, c) = trace_search(game, steps, &searches);
            (dt_ns, c)
        };
        (bf, benchmark)
    };

    let speedup = || {
        println!("Parallel");
        let (dt_ns_par, c_par) = benchmark_half(true, steps);
        println!();
        println!("Sequential");
        let (dt_ns_seq, c_seq) = benchmark_half(false, steps);
        println!();
        println!();
        println!("Parallel:   {:?} {}", c_par, c_par.summary(dt_ns_par));
        println!("Sequential: {:?} {}", c_seq, c_seq.summary(dt_ns_seq));
        let speedup = (dt_ns_seq as f64) / (dt_ns_par as f64);
        println!("Time Speedup: {speedup:.2}x");
        let rate_speedup =
            (c_par.states_visited as f64 * dt_ns_seq as f64) / (dt_ns_par as f64 * c_seq.states_visited as f64);
        println!("Rate Speedup: {rate_speedup:.2}x");
        println!("Parallel branching factor:   {:.2}", bf(c_par.states_visited as f64));
        println!("Sequential branching factor: {:.2}", bf(c_seq.states_visited as f64));
    };

    let benchmark = |parallel: bool, steps| {
        let s = if parallel { "Parallel" } else { "Sequential" };
        println!("{s}");
        let (dt_ns, c) = benchmark_half(parallel, steps);
        println!();
        println!();
        println!("{s}:   {:?} {}", c, c.summary(dt_ns));
        println!("{s} branching factor: {:.2}", bf(c.states_visited as f64));
    };

    let matches_started = AtomicI32::default();
    let do_match = |parallel: bool,
                    steps: u32,
                    rounds: u32,
                    get_standard_search_opts: &dyn Fn() -> SearchConfig|
     -> Result<(f32, Duration), std::io::Error> {
        let deck_opts = opts.deck();
        let t0 = Instant::now();
        let standard_opts = get_standard_search_opts();
        let (score, total_counter) = (0..rounds)
            .into_par_iter()
            .map(|_| {
                let i = matches_started.fetch_add(1, Ordering::SeqCst);
                let flip = i % 2 == 0;
                let mut search = ByPlayer(
                    deck_opts.make_search(parallel, deck_opts.get_limits()),
                    standard_opts.make_search(parallel, standard_opts.get_limits()),
                );
                if flip {
                    std::mem::swap(&mut search.0, &mut search.1);
                }
                let rng = SmallRng::seed_from_u64(deck_opts.seed.unwrap_or_default().overflowing_mul(i as u64).0);
                let game = deck_opts.get_standard_game(Some(rng)).unwrap();

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

        println!(
            "{:?}, rate={:.4}Mstates/s",
            total_counter,
            (total_counter.states_visited as f64) / (t0.elapsed().as_micros() as f64)
        );
        Ok(((score as f32) / ((2 * rounds) as f32), t0.elapsed()))
    };

    match opts {
        BenchmarkOpts::Speedup { .. } => speedup(),
        BenchmarkOpts::Benchmark { parallel, .. } => benchmark(parallel, steps),
        BenchmarkOpts::Evaluate { parallel, .. } => benchmark(parallel, 1),
        BenchmarkOpts::Match {
            parallel,
            steps,
            rounds,
            standard_algorithm,
            standard_time_limit_ms,
            ..
        } => {
            let (score, dt) = do_match(parallel, steps.unwrap_or(300), rounds.unwrap_or(100), &|| {
                standard_search_opts(standard_algorithm, standard_time_limit_ms)
            })?;
            println!("{score}, {:.2}ms", dt.as_millis());
        }
    };

    Ok(())
}

fn get_winner_value(winner: Option<PlayerId>, flip: bool) -> (&'static str, i32) {
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
