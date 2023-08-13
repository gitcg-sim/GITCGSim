use gitcg_sim::deck::cli_args::{GenericSearch, SearchAlgorithm, SearchConfig};
use gitcg_sim::types::by_player::ByPlayer;
use gitcg_sim::types::game_state::PlayerId;
use gitcg_sim::types::nondet::NondetState;
use instant::Duration;
use lazy_static::__Deref;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;
use std::ops::Add;
use std::rc::Rc;
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
    #[structopt(help = "Measure win rate against a standarized opponent.")]
    Match {
        #[structopt(long)]
        parallel: bool,

        #[structopt(long)]
        steps: Option<u32>,

        #[structopt(long)]
        rounds: Option<u32>,

        #[structopt(flatten)]
        deck: DeckOpts,

        #[structopt(long)]
        standard_algorithm: Option<SearchAlgorithm>,

        #[structopt(long)]
        standard_time_limit_ms: Option<u128>,
    },
}

impl BenchmarkOpts {
    fn deck(&self) -> &DeckOpts {
        match self {
            BenchmarkOpts::Speedup { deck, .. } => deck,
            BenchmarkOpts::Benchmark { deck, .. } => deck,
            BenchmarkOpts::Evaluate { deck, .. } => deck,
            BenchmarkOpts::Match { deck, .. } => deck,
        }
    }
}

fn trace_search<S: NondetState + Serialize>(
    game: &GameStateWrapper<S>,
    steps: u32,
    search: &mut GenericSearch<S>,
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
        let SearchResult {
            pv,
            eval: v,
            counter: c,
        } = {
            let mut game1 = game.clone();
            game1.hide_private_information(p.opposite());
            search.search(&game1, p)
        };
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

fn match_round<S: NondetState + Serialize>(
    mut game: GameStateWrapper<S>,
    search: &mut ByPlayer<GenericSearch<S>>,
    steps: u32,
) -> (Option<PlayerId>, Duration, SearchCounter) {
    let t0 = Instant::now();
    let mut total_counter = SearchCounter::default();
    for _ in 0..steps {
        if game.winner().is_some() {
            break;
        }

        let player_id_to_move = game.to_move().unwrap();
        let search = &mut search[player_id_to_move];
        let (
            game0,
            root_hash_0,
            SearchResult {
                pv,
                eval: _,
                counter: c,
            },
        ) = {
            let game1 = game.clone();
            let mut game1 = game.clone();
            game1.hide_private_information(player_id_to_move.opposite());
            let h = game1.zobrist_hash();
            (game1.clone(), h, search.search(&game1, player_id_to_move))
        };
        total_counter.add_in_place(&c);
        if pv.is_empty() {
            println!("perform_match: PV is empty.");
            break;
        }
        let input = pv.head().unwrap();
        let acts = game.actions();
        if !acts.iter().copied().any(|a| a == input) {
            println!("----------");
            let mcts = search.get_mcts().unwrap();
            let (root_hash_1, root_token) = mcts.root.unwrap();
            let root_node = mcts.tree.get(root_token).unwrap();
            let root_actions = root_node
                .children(&mcts.tree)
                .map(|n| n.data.action.unwrap())
                .collect::<Vec<_>>();
            let root_hash_2 = root_node.data.state.zobrist_hash();
            println!("h0={root_hash_0} h1={root_hash_1}, h_root={root_hash_2}");
            println!("rA={root_actions:?}");
            println!("Input: {input:?}");
            // println!("PV: {:?}", pv.into_iter().collect::<Vec<_>>());
            println!(
                "Actions (external): [{:?}] {:?} contains={}",
                game.zobrist_hash(),
                acts,
                acts.iter().copied().any(|a| a == input)
            );
            println!("Root actions (tree): [{:?}] {:?}", game.zobrist_hash(), root_actions);
            println!(
                "Root actions (reeval): [{:?}] {:?}",
                root_node.data.state.zobrist_hash(),
                root_node.data.state.actions().into_iter().collect::<Vec<_>>()
            );
            println!("----------");
            println!(
                "A0: {} {:?}",
                root_hash_0,
                game0.actions().into_iter().collect::<Vec<_>>()
            );
            println!(
                "A1: {} {:?}",
                game.zobrist_hash(),
                game.actions().into_iter().collect::<Vec<_>>()
            );
            println!(
                "A2: {} {:?}",
                root_node.data.state.zobrist_hash(),
                root_node.data.state.actions().into_iter().collect::<Vec<_>>()
            );
            let s3 = root_node.data.state.clone();
            println!(
                "A3: {} {:?}",
                s3.zobrist_hash(),
                s3.actions().into_iter().collect::<Vec<_>>()
            );
            println!(
                "A4: {} {:?}",
                s3.zobrist_hash(),
                s3.actions().into_iter().collect::<Vec<_>>()
            );
            println!("----------");
            println!("Game state 0: {:#?}", &game0.game_state.players.1.dice.clone());
            println!("----------");
            println!("Game state 1: {:#?}", game.clone().game_state.players.1.dice.clone());
            println!("----------");
            println!(
                "Game state 2: {:#?}",
                root_node.data.state.game_state.players.1.dice.clone()
            );
            println!("----------");
            // let json = serde_json::to_string(&game0).unwrap();
            // println!("{json}");
            std::process::exit(2);
            // panic!("perform_match: Action is not available");
        }
        if let Err(e) = game.advance(input) {
            println!("----------");
            println!("Error: {e:?}");
            println!("Input: {input:?}");
            println!(
                "Actions: {:?} contains={}",
                acts,
                acts.iter().copied().any(|a| a == input)
            );
            println!("Game state: {game:#?}");
            println!("----------");
            println!();
            panic!("Exit");
        };
    }

    (game.winner(), t0.elapsed(), total_counter)
}

fn standard_search_opts(algorithm: Option<SearchAlgorithm>, standard_time_limit_ms: Option<u128>) -> SearchConfig {
    SearchConfig {
        algorithm,
        mcts_c: Some(2.0),
        mcts_b: None,
        mcts_random_playout_iters: Some(10),
        mcts_random_playout_max_steps: Some(20),
        mcts_random_playout_bias: Some(50.0),
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
    let (bf, benchmark) = {
        let deck_opts: &DeckOpts = opts.deck();
        let search_opts = &deck_opts.search;
        let depth: u8 = search_opts.search_depth.unwrap_or(8);
        let bf = move |n: f64| n.powf(1_f64 / (depth as f64));
        let game = Rc::new(deck_opts.get_standard_game(None)?);
        let benchmark = move |parallel: bool, steps: u32| {
            let mut search = deck_opts.make_search(parallel, deck_opts.get_limits());
            let game = game.deref();
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

    let do_match = |parallel: bool,
                    steps: u32,
                    rounds: u32,
                    get_standard_search_opts: &dyn Fn() -> SearchConfig|
     -> Result<(f32, Duration), std::io::Error> {
        let deck_opts = opts.deck();
        let t0 = Instant::now();
        let opts = get_standard_search_opts();
        let (score, total_counter) = (0..rounds)
            .into_par_iter()
            .map(|i| {
                let mut search = ByPlayer(
                    deck_opts.make_search(parallel, deck_opts.get_limits()),
                    opts.make_search(parallel, opts.get_limits()),
                );
                let rng = SmallRng::seed_from_u64(deck_opts.seed.unwrap_or_default().overflowing_mul(i as u64).0);
                let game = deck_opts.get_standard_game(Some(rng)).unwrap();
                println!("Round {:3}", i + 1);
                let (winner, dt, c) = match_round(game, &mut search, steps);
                println!("Round {:3} ... {winner:?} {:.2}ms", i + 1, dt.as_millis());
                let d_score = match winner {
                    Some(PlayerId::PlayerFirst) => 2,
                    Some(PlayerId::PlayerSecond) => 0,
                    None => 1,
                };
                (d_score, c)
            })
            .reduce(|| (0, Default::default()), |(s0, c0), (s1, c1)| (s0 + s1, c0.add(c1)));
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
            let (score, dt) = do_match(parallel, steps.unwrap_or(200), rounds.unwrap_or(100), &|| {
                standard_search_opts(standard_algorithm, standard_time_limit_ms)
            })?;
            println!("{score}, {:.2}ms", dt.as_millis());
        }
    };

    Ok(())
}
