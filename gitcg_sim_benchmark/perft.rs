use gitcg_sim::prelude::*;
use gitcg_sim_cli_utils::cli_args::SearchOpts;
use instant::Instant;

fn perft_serial<S: NondetState>(gs: &GameStateWrapper<S>, depth: u8) -> u64 {
    if depth == 0 {
        return 0;
    }

    let actions = gs.actions();
    let mut n = actions.len() as u64;
    if depth == 1 {
        return n;
    }
    for action in actions {
        let mut gs1 = gs.clone();
        gs1.advance(action).expect("Failed to advance.");
        n += perft_serial(&gs1, depth - 1);
    }
    n
}

fn perft_parallel<S: NondetState>(gs: &GameStateWrapper<S>, depth: u8, parallel_depth: u8) -> u64 {
    use rayon::prelude::*;
    if parallel_depth == 0 || depth <= 2 {
        return perft_serial(gs, depth);
    }
    let actions = gs.actions();
    let n = actions.len() as u64;
    let n1: u64 = actions
        .into_par_iter()
        .map(|&action| {
            let mut gs1 = gs.clone();
            gs1.advance(action).expect("Failed to advance.");
            perft_parallel(&gs1, depth - 1, parallel_depth - 1)
        })
        .sum();
    n + n1
}

pub fn run_perft(opts: &SearchOpts, parallel: bool, depth: u8) -> Result<(), std::io::Error> {
    let gs = opts.get_standard_game(None)?;
    let start_time = Instant::now();
    let n = if parallel {
        perft_parallel(&gs, depth, 4)
    } else {
        perft_serial(&gs, depth)
    };
    let dt = start_time.elapsed();
    let dt_ms = (dt.as_nanos() as f64) * 1e-6;
    let rate = 1e-3f64 * (n as f64) / dt_ms;
    println!("depth = {depth}, n_pos = {n:.3}, dt = {dt_ms:.3} ms, rate = {rate:.3} Mactions/s");
    Ok(())
}
