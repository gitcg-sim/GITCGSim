use std::sync::mpsc::channel;
use std::{
    borrow::Borrow,
    cell::RefCell,
    ops::ControlFlow,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::Sender,
        Arc,
    },
};

use dfdx::{optim::Sgd, prelude::*};
use gitcg_sim::rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
use gitcg_sim_search::training::features::game_state_features;
use gitcg_sim_search::training::features::input_features::input_features;
use gitcg_sim_search::{
    mcts::{MCTSConfig, SelfPlayDataPoint, MCTS},
    playout::Playout,
    prelude::*,
    training::{
        as_slice::*,
        features::{Features, InputFeatures},
        policy::*,
    },
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;
use serde_json::json;
use structopt::StructOpt;

use ndarray::Array1;

use gitcg_sim::prelude::*;
use gitcg_sim_cli_utils::cli_args::SearchOpts;
use gitcg_sim_search::training::{eval::*, policy::N_IN};

#[derive(Debug, StructOpt, Copy, Clone)]
pub struct Regularization {
    #[structopt(long = "--l1-regularization", help = "L1 regularization coefficient")]
    pub l1: Option<f32>,
    #[structopt(long = "--l2-regularization", help = "L2 regularization coefficient")]
    pub l2: Option<f32>,
}

#[derive(Debug, StructOpt, Clone)]
pub struct TDLOpts {
    #[structopt(short = "N", long = "--max-iters", help = "Max. Iterations", default_value = "10000")]
    pub max_iters: u32,
    #[structopt(
        short = "L",
        long = "--log-iters",
        help = "Iterations per log print",
        default_value = "500"
    )]
    pub log_iters: u32,
    #[structopt(long, help = "Base learning rate", default_value = "1.0")]
    pub base_learning_rate: f32,
    #[structopt(
        long,
        help = "Number of learning rounds before learning rate reduces by 5%",
        default_value = "1000"
    )]
    pub learning_rate_decay: i32,
    #[structopt(
        long,
        help = "Temporal difference learning discounting rate per move",
        default_value = "0.95"
    )]
    pub temporal_rate: f32,
    #[structopt(
        long,
        help = "Boltzmann action selection - reciprocal of temperature",
        default_value = "1"
    )]
    pub beta: f32,
    #[structopt(flatten)]
    pub regularization: Regularization,
}

#[derive(Debug, StructOpt, Clone)]
pub struct PolicyOpts {
    #[structopt(default_value = "250", long = "--mcts-time-limit-ms")]
    pub mcts_time_limit_ms: f32,
    #[structopt(
        default_value = "4",
        long = "--min-depth",
        help = "Search depth condition to include as training data. Negative: at least abs(min depth) + PV depth, Positive: depth >= min depth, Zero: include all depths"
    )]
    pub mcts_min_depth: i8,
    #[structopt(long = "--l2-regularization", help = "L2 regularization coefficient")]
    pub l2_regularization: Option<f64>,
    #[structopt(long = "--save-npz")]
    pub save_npz: Option<std::path::PathBuf>,
    #[structopt(long = "--load-npz")]
    pub load_npz: Option<std::path::PathBuf>,
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Genius Invokation TCG simulator - self-play")]
pub enum SelfPlayOpts {
    #[structopt(help = "Run temporal difference learning for evaluation")]
    TDL {
        #[structopt(flatten)]
        search: SearchOpts,
        #[structopt(flatten)]
        tdl: TDLOpts,
    },
    #[structopt(help = "Run policy learning for policy")]
    Policy {
        #[structopt(flatten)]
        search: SearchOpts,
        #[structopt(flatten)]
        policy: PolicyOpts,
    },
}

/// Generate a playout using a search. Ends at game end (`Break(..)` returned, winner decided or `max_iters` reached).
fn run_playout<
    T: GameTreeSearch<GameStateWrapper<S>>,
    S: NondetState,
    C: FnMut(&ByPlayer<T>, &GameStateWrapper<S>, Input) -> ControlFlow<()>,
>(
    initial: &GameStateWrapper<S>,
    models: &RefCell<ByPlayer<T>>,
    max_iters: usize,
    mut on_step: C,
) -> Vec<GameStateWrapper<S>> {
    let mut states: Vec<GameStateWrapper<S>> = Vec::with_capacity(16);
    states.push(initial.clone());
    let it = Playout::new(max_iters, initial.clone(), models);
    for res in it.iter_playout() {
        let (input, game_state, models) = res.unwrap();
        states.push(game_state.clone());
        match on_step(&models.borrow(), &game_state, input) {
            ControlFlow::Continue(_) => {}
            ControlFlow::Break(_) => break,
        }
    }
    states
}

fn winner_eval(winner: PlayerId, player_id: PlayerId) -> (f32, Features) {
    (
        if winner == player_id {
            SelfPlayModel::WIN
        } else {
            SelfPlayModel::LOSE
        },
        Default::default(),
    )
}

#[derive(Serialize)]
struct DebugEntry {
    pub iter: u32,
    pub learning_rate: f32,
    pub player_id: PlayerId,
    #[serde(flatten)]
    pub game_state: Features,
}

fn run_tdl_self_play<
    T: GameTreeSearch<GameStateWrapper<S>> + GetSelfPlayModel,
    S: NondetState,
    F: Fn(PlayerId, Features) -> DebugEntry,
>(
    initial: &GameStateWrapper<S>,
    searches: &RefCell<ByPlayer<T>>,
    learning_rate: f32,
    temporal_rate: f32,
    regularization: &Regularization,
    make_debug_entry: Option<F>,
) {
    let states = run_playout(initial, searches, 300, |_, _, _| ControlFlow::Continue(()));
    let mut searches = searches.borrow_mut();
    let evals = ByPlayer::generate(|player_id| {
        let eval_game_state = |game_state: &GameStateWrapper<S>| {
            if let Some(winner) = game_state.winner() {
                winner_eval(winner, player_id)
            } else {
                searches.get(player_id).self_play_model().evaluate(game_state)
            }
        };
        states.iter().map(eval_game_state).collect::<Vec<_>>()
    });

    if let Some(make_debug_entry) = make_debug_entry {
        for player_id in PlayerId::VALUES {
            let model = searches[player_id].self_play_model();
            let entry = make_debug_entry(player_id, model.weights);
            println!("{}", serde_json::to_string(&entry).unwrap_or_default());
        }
    }

    for player_id in PlayerId::VALUES {
        let model = searches[player_id].self_play_model_mut();
        let weights = model.weights.as_slice_mut();
        let n_states = states.len();
        let evals = &evals[player_id];
        for t in 0..(n_states - 1) {
            let mut grad = evals[t].1.as_slice();
            let sum_td: f32 = (t..(n_states - 1))
                .map(|j| {
                    let temporal_diff = evals[j + 1].0 - evals[j].0;
                    temporal_rate.powi((j - t) as i32) * temporal_diff
                })
                .sum();

            if let Some(a) = regularization.l1 {
                for (gi, wi) in grad.iter_mut().zip(weights.iter().copied()) {
                    *gi += a * wi.abs();
                }
            }

            if let Some(a) = regularization.l2 {
                for (gi, wi) in grad.iter_mut().zip(weights.iter().copied()) {
                    *gi += 2.0 * a * wi;
                }
            }

            let a = sum_td * learning_rate;
            for (wi, gi) in weights.iter_mut().zip(grad.iter().copied()) {
                *wi += a * gi;
            }
        }
    }
}

fn new_search(init_weights: Features, player_id: PlayerId, beta: f32, delta: f32) -> SelfPlaySearch {
    SelfPlaySearch::new(init_weights, player_id, beta, delta)
}

fn main_tdl(mut deck: SearchOpts, opts: TDLOpts) -> Result<(), std::io::Error> {
    let mut seed_gen = thread_rng();
    let (beta, delta) = (opts.beta, 1e-4);
    let searches = RefCell::new(ByPlayer::generate(|p| new_search(Default::default(), p, beta, delta)));
    for i in 0..=opts.max_iters {
        deck.seed = Some(seed_gen.gen());
        let initial = deck.standard_game(Some(SmallRng::from_seed(seed_gen.gen())))?;
        let learning_rate = opts.base_learning_rate * 0.95f32.powi((i as i32) / opts.learning_rate_decay);
        let debug = if i % opts.log_iters == 0 {
            Some(|player_id, game_state| DebugEntry {
                iter: i,
                learning_rate,
                player_id,
                game_state,
            })
        } else {
            None
        };
        run_tdl_self_play(
            &initial,
            &searches,
            learning_rate,
            opts.temporal_rate,
            &opts.regularization,
            debug,
        );
    }
    Ok(())
}

fn generate_data_points_mcts<I: FnMut() -> GameStateWrapper<S>, S: NondetState>(
    tx: Sender<(Features, InputFeatures<f32>, u8)>,
    mut initial: I,
    searches: &RefCell<ByPlayer<MCTS<GameStateWrapper<S>>>>,
    mcts_min_depth: i8,
    data_points_per_iter: usize,
    iterations: usize,
) {
    let mut rng = thread_rng();
    let mut initial = initial();
    for i in 0..iterations {
        println!(
            "{}",
            json!({ "gen_iter": i, "new_playout": true, "initial_state": initial.game_state })
        );
        let game_states = run_playout(&initial, searches, 300, |searches, _game_state, _input| {
            let player_id = PlayerId::PlayerFirst;
            let mcts: &MCTS<GameStateWrapper<S>> = searches.get(player_id);
            let should_include = |depth: u8, pv_depth: usize| -> bool {
                match mcts_min_depth {
                    0 => true,
                    _ if mcts_min_depth < 0 => (depth as usize) >= pv_depth - ((-mcts_min_depth) as usize),
                    _ => depth >= (mcts_min_depth as u8),
                }
            };
            let mut vec: Vec<SelfPlayDataPoint<GameStateWrapper<S>>> = Default::default();
            let push_data_point = |x| {
                vec.push(x);
                ControlFlow::Continue(())
            };
            mcts.self_play_policy_data_points(player_id, should_include, push_data_point);
            vec.sort_by_key(|d| 1_0000i32 - 1000 * (d.depth as i32) + rng.gen_range(0..10));
            vec.truncate(data_points_per_iter);
            for SelfPlayDataPoint {
                state: gs,
                action_weights,
                depth,
                ..
            } in vec
            {
                let mut weighted_features = <InputFeatures<f32> as AsSlice<f32>>::Slice::default();
                let avg_w =
                    action_weights.iter().copied().map(|(_, w)| w).sum::<f32>() / (weighted_features.len() as f32);
                for (i, wi) in weighted_features.iter_mut().enumerate() {
                    let mut unit: <InputFeatures<f32> as AsSlice<f32>>::Slice = Default::default();
                    unit[i] = 1.0;
                    let mut tot_dot = 0.0;
                    let mut tot_weight = 0.0;
                    for (act, weight) in action_weights.iter().copied() {
                        let input_features = input_features(act, 1.0).as_slice();
                        let dot: f32 = unit.iter().copied().zip(input_features).map(|(a, b)| a * b).sum();
                        tot_weight += dot * weight;
                        tot_dot += dot;
                    }
                    *wi = if tot_dot < 1e-3 { avg_w } else { tot_weight / tot_dot };
                }

                let game_state_features = game_state_features::features(&gs.game_state);
                tx.send((game_state_features, InputFeatures::from_slice(weighted_features), depth))
                    .unwrap();
            }
            ControlFlow::Continue(())
        });
        if game_states.is_empty() {
            continue;
        }
        let last_game_state = game_states.last().unwrap().clone();
        if last_game_state.winner().is_none() {
            initial = last_game_state;
        } else {
            initial = initial();
        }
    }
}

fn main_policy(deck: SearchOpts, opts: PolicyOpts) -> Result<(), std::io::Error> {
    const BATCH_SIZE: usize = 500;
    const CUTOFF_PER_ITER: usize = 5;
    let config = MCTSConfig {
        cpuct: deck.search.cpuct_config(),
        random_playout_iters: deck.search.mcts_random_playout_iters.unwrap_or(1),
        random_playout_bias: deck.search.mcts_random_playout_bias,
        random_playout_cutoff: deck.search.mcts_random_playout_max_steps.unwrap_or(20),
        policy_bias: deck.search.mcts_policy_bias,
        tt_size_mb: deck.search.tt_size_mb.unwrap_or(32),
        limits: Some(SearchLimits {
            max_time_ms: Some(opts.mcts_time_limit_ms as u128),
            max_positions: None,
        }),
        debug: false,
        parallel: true,
    };

    let games = AtomicU32::new(0);
    let (tx, rx) = channel();
    let run_gen = {
        |tx: Sender<_>| {
            let config = &config;
            let deck = Arc::new(deck.clone());
            let games = &games;
            let initial = move || {
                let deck: &SearchOpts = Arc::borrow(&deck);
                let mut deck = deck.clone();
                let mut seed_gen = thread_rng();
                seed_gen.gen_range(0..255);
                deck.seed = Some(seed_gen.gen());
                games.fetch_add(1, Ordering::AcqRel);
                deck.standard_game(Some(SmallRng::from_seed(seed_gen.gen()))).unwrap()
            };
            let searches = RefCell::new(ByPlayer::generate(move |_| MCTS::new(*config)));
            generate_data_points_mcts(tx, initial, &searches, opts.mcts_min_depth, CUTOFF_PER_ITER, 1_000_000)
        }
    };
    let games = &games;
    let run_learning = move || {
        let mut total_data_points = 0usize;
        let mut model: PolicyNetwork = PolicyNetwork::default();
        if let Some(npz_path) = &opts.load_npz {
            if let Err(e) = model.load_from_npz(npz_path) {
                println!("{}", json!({ "load_error": format!("{e}") }));
            };
        }
        let mut grads = model.alloc_grads();
        let mut opt = Sgd::new(
            &model.model,
            SgdConfig {
                lr: 1e-2,
                momentum: Some(Momentum::Classic(0.5)),
                weight_decay: opts.l2_regularization.map(WeightDecay::L2),
                // ..Default::default()
            },
        );

        for iter in 0..10_000 {
            let mut data_points: Vec<(Features, InputFeatures<f32>, u8)> = vec![];
            while data_points.len() < BATCH_SIZE {
                data_points.push(rx.recv().unwrap());
            }

            let batch = (0..BATCH_SIZE).map(|_| data_points.pop().unwrap()).collect::<Vec<_>>();
            let (inputs, outputs): (Vec<_>, Vec<_>) = batch
                .iter()
                .take(BATCH_SIZE)
                .map(|(x, y, _)| (x.as_slice(), y.as_slice()))
                .unzip();
            let (inputs, outputs) = (inputs.concat(), outputs.concat());
            total_data_points += batch.len();

            fn t_batched<const L: usize>(inputs: Vec<f32>) -> Array1<f32> {
                Array1::from_shape_vec(BATCH_SIZE * L, inputs).unwrap()
                // Array2::from_shape_vec((BATCH_SIZE, L), inputs)
                //     .unwrap()
                //     .t()
                //     .to_owned()
                //     .into_shape((BATCH_SIZE * L,))
                //     .unwrap()
            }

            let input_data = t_batched::<N_IN>(inputs);
            let output_data = t_batched::<N_OUT>(outputs);

            let dev = &model.dev;
            let x = {
                let mut x = model.alloc_x::<BATCH_SIZE>();
                x.copy_from(input_data.as_slice().unwrap());
                x
            };
            let y = model.model.forward_mut(x.traced(grads));
            let mut y0: Tensor<Rank2<BATCH_SIZE, N_OUT>, f32, _> = dev.zeros();
            y0.copy_from(output_data.as_slice().unwrap());
            let loss = (y - y0).square().mean();
            let loss_value = loss.array();
            grads = loss.backward();
            opt.update(&mut model.model, &grads).unwrap();
            let games = games.load(Ordering::Acquire);
            println!(
                "{}",
                json!({
                    "loss": loss_value,
                    "iter": iter,
                    "total_data_points": total_data_points,
                    "games": games,
                })
            );
            if let Some(npz_path) = &opts.save_npz {
                model.save_npz(npz_path).unwrap();
            }
        }
    };
    let tx = &tx;
    rayon::join(
        || {
            (0..(rayon::max_num_threads() - 1).max(1))
                .into_par_iter()
                .for_each(|_| run_gen(tx.clone()))
        },
        run_learning,
    );
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let opts = SelfPlayOpts::from_args();
    match opts {
        SelfPlayOpts::TDL { search: deck, tdl } => main_tdl(deck, tdl),
        SelfPlayOpts::Policy { search: deck, policy } => main_policy(deck, policy),
    }
}
