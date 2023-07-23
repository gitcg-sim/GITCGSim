use gitcg_sim::deck::cli_args::GenericSearch;
use gitcg_sim::deck::random_decklist;
use gitcg_sim::types::nondet::NondetState;
use rand::prelude::*;
use std::time::Instant;
use structopt::StructOpt;

use gitcg_sim::deck::cli_args::DeckOpt;
use gitcg_sim::game_tree_search::*;

fn trace_search<S: NondetState>(
    mut game: GameStateWrapper<S>,
    steps: u32,
    search: &mut GenericSearch<S>,
) -> (u128, SearchCounter) {
    // game.convert_to_tactical_search();
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
    let opt = DeckOpt::from_args();
    let (decklist1, decklist2) = if opt.random_decks {
        let mut r = SmallRng::seed_from_u64(opt.seed.unwrap_or(100));
        r.gen_bool(0.5);
        (random_decklist(&mut r), random_decklist(&mut r))
    } else {
        (opt.get_player1_deck()?, opt.get_player2_deck()?)
    };
    let depth: u8 = opt.search_depth.unwrap_or(8);
    let steps: u32 = opt.steps.unwrap_or(200);
    let benchmark = move |parallel: bool| {
        let game = new_standard_game(&decklist1, &decklist2, SmallRng::seed_from_u64(opt.seed.unwrap_or(100)));
        let mut search = opt.make_search(parallel);
        let (dt_ns, c) = trace_search(game, steps, &mut search);
        (dt_ns, c)
    };

    println!("Parallel");
    let (dt_ns_par, c_par) = benchmark(true);
    println!();
    println!("Sequential");
    let (dt_ns_seq, c_seq) = benchmark(false);
    println!();
    println!();
    println!("Parallel:   {:?} {}", c_par, c_par.summary(dt_ns_par));
    println!("Sequential: {:?} {}", c_seq, c_seq.summary(dt_ns_seq));
    let speedup = (dt_ns_seq as f64) / (dt_ns_par as f64);
    println!("Time Speedup: {speedup:.2}x");
    let rate_speedup =
        (c_par.states_visited as f64 * dt_ns_seq as f64) / (dt_ns_par as f64 * c_seq.states_visited as f64);
    println!("Rate Speedup: {rate_speedup:.2}x");
    let bf_par = f64::powf(c_par.states_visited as f64, 1_f64 / (depth as f64));
    println!("Parallel branching factor:   {bf_par:.2}");
    let bf_seq = f64::powf(c_seq.states_visited as f64, 1_f64 / (depth as f64));
    println!("Sequential branching factor: {bf_seq:.2}");

    Ok(())
}
