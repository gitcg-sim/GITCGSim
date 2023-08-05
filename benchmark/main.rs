use gitcg_sim::deck::cli_args::GenericSearch;
use gitcg_sim::deck::random_decklist;
use gitcg_sim::types::nondet::NondetState;
use rand::prelude::*;
use std::time::Instant;
use structopt::StructOpt;

use gitcg_sim::deck::cli_args::DeckOpts;
use gitcg_sim::game_tree_search::*;

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Genius Invokation TCG simulator")]
pub enum BenchmarkOpts {
    #[structopt(help = "Run a game in parallel execution then in sequential execution, and compare speedups.")]
    Speedup {
        #[structopt(flatten)]
        deck: DeckOpts,
    },
    #[structopt(help = "Run a game in parallel or sequential execution.")]
    Benchmark {
        #[structopt(long)]
        parallel: bool,
        #[structopt(flatten)]
        deck: DeckOpts,
    },
    #[structopt(help = "Evaluate the first move of a position.")]
    Evaluate {
        #[structopt(long)]
        parallel: bool,
        #[structopt(flatten)]
        deck: DeckOpts,
    },
}
impl BenchmarkOpts {
    fn deck(&self) -> &DeckOpts {
        match self {
            BenchmarkOpts::Speedup { deck, .. } => deck,
            BenchmarkOpts::Benchmark { deck, .. } => deck,
            BenchmarkOpts::Evaluate { deck, .. } => deck,
        }
    }
}

fn trace_search<S: NondetState>(
    mut game: GameStateWrapper<S>,
    steps: u32,
    search: &mut GenericSearch<S>,
) -> (u128, SearchCounter) {
    let mut total_counter = SearchCounter::default();
    let mut total_time: u128 = 0;
    for i in 0..steps {
        let t1 = Instant::now();
        if let Some(winner) = game.winner() {
            println!("Winner: {winner:?}");
            break;
        }
        let p = game.to_move().unwrap();
        let mut game1 = game.clone();
        game1.hide_private_information(p.opposite());
        let SearchResult {
            pv,
            eval: v,
            counter: c,
        } = search.search(&game1, p);
        total_counter.add_in_place(&c);
        let dt_ns = t1.elapsed().as_nanos();
        total_time += dt_ns;
        if pv.is_empty() {
            panic!("PV is empty.");
        }
        let input = pv.head().unwrap();
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

fn main() -> Result<(), std::io::Error> {
    let opts = BenchmarkOpts::from_args();
    let steps: u32 = opts.deck().steps.unwrap_or(200);
    let (bf, benchmark) = {
        let opt: &DeckOpts = opts.deck();
        let search_opts = &opt.search;
        let (decklist1, decklist2) = if opt.random_decks {
            let mut r = SmallRng::seed_from_u64(opt.seed.unwrap_or(100));
            r.gen_bool(0.5);
            (random_decklist(&mut r), random_decklist(&mut r))
        } else {
            (opt.get_player1_deck()?, opt.get_player2_deck()?)
        };

        let depth: u8 = search_opts.search_depth.unwrap_or(8);
        let bf = move |n: f64| n.powf(1_f64 / (depth as f64));
        let benchmark = move |parallel: bool, steps: u32| {
            let game = new_standard_game(&decklist1, &decklist2, SmallRng::seed_from_u64(opt.seed.unwrap_or(100)));
            let mut search = opt.make_search(parallel, opt.get_limits());
            let (dt_ns, c) = trace_search(game, steps, &mut search);
            (dt_ns, c)
        };
        (bf, benchmark)
    };

    let speedup = || {
        println!("Parallel");
        let (dt_ns_par, c_par) = benchmark(true, steps);
        println!();
        println!("Sequential");
        let (dt_ns_seq, c_seq) = benchmark(false, steps);
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
        let (dt_ns, c) = benchmark(parallel, steps);
        println!();
        println!();
        println!("{s}:   {:?} {}", c, c.summary(dt_ns));
        println!("{s} branching factor: {:.2}", bf(c.states_visited as f64));
    };

    match opts {
        BenchmarkOpts::Speedup { .. } => speedup(),
        BenchmarkOpts::Benchmark { parallel, .. } => benchmark(parallel, steps),
        BenchmarkOpts::Evaluate { parallel, .. } => benchmark(parallel, 1),
    };

    Ok(())
}
