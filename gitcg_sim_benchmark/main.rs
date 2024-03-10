use instant::Duration;
use lazy_static::__Deref;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use structopt::StructOpt;

use gitcg_sim::prelude::*;
use gitcg_sim_cli_utils::cli_args::{DeckGen, SearchAlgorithm, SearchConfig, SearchOpts};
use gitcg_sim_search::mcts::CpuctConfig;

mod match_round;
use match_round::*;

mod compare;
use compare::*;

mod perft;
use perft::*;

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
        #[structopt(long = "--parallel")]
        parallel: bool,
        #[structopt(flatten)]
        search: SearchOpts,
    },
    #[structopt(help = "Evaluate the first move of a position.")]
    Evaluate {
        #[structopt(long = "--parallel")]
        parallel: bool,
        #[structopt(flatten)]
        search: SearchOpts,
    },
    #[structopt(help = "Measure win rate against a standarized opponent.")]
    Match {
        #[structopt(long = "--parallel")]
        parallel: bool,

        #[structopt(long)]
        match_steps: Option<u32>,

        #[structopt(long)]
        rounds: Option<u32>,

        #[structopt(flatten)]
        search: SearchOpts,

        #[structopt(long)]
        standard_algorithm: Option<SearchAlgorithm>,

        #[structopt(long)]
        standard_time_limit_ms: Option<u128>,
    },
    #[structopt(help = "Perform head-to-head matches between multiple search configurations.")]
    Compare {
        #[structopt(help = "Path to the JSON file for the configuration.")]
        json_path: PathBuf,
    },
    #[structopt(
        help = "Count the total number of positions visited at a given depth, starting from a particular initial position."
    )]
    Perft {
        #[structopt(long = "--parallel")]
        parallel: bool,

        #[structopt(long = "--iterative", help = "Run perft for each depth starting from 1.")]
        iterative: bool,

        #[structopt(flatten)]
        search: SearchOpts,
    },
}

lazy_static! {
    pub static ref _DEFAULT_SEARCH_OPTS: SearchOpts = SearchOpts {
        deck: DeckGen {
            random_decks: true,
            ..Default::default()
        },
        ..Default::default()
    };
}

impl BenchmarkOpts {
    fn deck(&self) -> Option<&SearchOpts> {
        match self {
            BenchmarkOpts::Speedup { search: deck, .. }
            | BenchmarkOpts::Benchmark { search: deck, .. }
            | BenchmarkOpts::Evaluate { search: deck, .. }
            | BenchmarkOpts::Match { search: deck, .. } => Some(deck),
            BenchmarkOpts::Compare { .. } => Some(&_DEFAULT_SEARCH_OPTS),
            BenchmarkOpts::Perft { search: deck, .. } => Some(deck),
        }
    }
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
    let steps: u32 = opts.deck().and_then(|x| x.steps).unwrap_or(200);
    let (bf, benchmark_half) = {
        let deck_opts: &SearchOpts = opts.deck().expect("Deck is expected.");
        let search_opts = &deck_opts.search;
        let depth: u8 = search_opts.search_depth.unwrap_or(8);
        let bf = move |n: f64| n.powf(1_f64 / (depth as f64));
        let game = Rc::new(deck_opts.standard_game(None)?);
        let benchmark = move |parallel: bool, steps: u32| {
            let mut searches = ByPlayer::generate(|_| deck_opts.make_search(parallel, deck_opts.limits()));
            let game = game.deref();
            let (dt_ns, c) = trace_search(game, &mut searches, steps);
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

    let do_match = |parallel: bool,
                    steps: u32,
                    rounds: u32,
                    standard_search_opts: &dyn Fn() -> SearchConfig|
     -> Result<(f32, Duration), std::io::Error> {
        let deck_opts = opts.deck().expect("Deck is expected.");
        let t0 = Instant::now();
        let standard_opts = standard_search_opts();
        let make_search = || {
            ByPlayer(
                deck_opts.make_search(parallel, deck_opts.limits()),
                standard_opts.make_search(parallel, standard_opts.limits()),
            )
        };
        let (_, score, total_counter) = iterate_match(
            &make_search,
            &|rng| {
                deck_opts
                    .standard_game(Some(rng))
                    .expect("Failed to create initial game state.")
            },
            IterateMatchOpts {
                rounds,
                random_seed: deck_opts.seed.unwrap_or(100),
                steps,
            },
        );

        println!(
            "{:?}, rate={:.4}Mstates/s",
            total_counter,
            (total_counter.states_visited as f64) / (t0.elapsed().as_micros() as f64)
        );
        Ok((score, t0.elapsed()))
    };

    match opts {
        BenchmarkOpts::Speedup { .. } => speedup(),
        BenchmarkOpts::Benchmark { parallel, .. } => benchmark(parallel, steps),
        BenchmarkOpts::Evaluate { parallel, .. } => benchmark(parallel, 1),
        BenchmarkOpts::Match {
            parallel,
            match_steps: steps,
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
        BenchmarkOpts::Compare { json_path, .. } => {
            let opts = parse_compare_opts(&json_path).expect("Failed to parse config.");
            if let Err(e) = main_compare(opts) {
                println!("{e}");
                std::process::exit(1)
            }
        }
        BenchmarkOpts::Perft {
            parallel,
            iterative,
            search,
        } => {
            let depth = search.search.search_depth.unwrap_or(3);
            if iterative {
                for depth in 1..=depth {
                    run_perft(&search, parallel, depth)?
                }
            } else {
                run_perft(&search, parallel, search.search.search_depth.unwrap_or(3))?
            }
        }
    };

    Ok(())
}
