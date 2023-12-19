use std::{cell::RefCell, ops::ControlFlow};

use dfdx::{optim::Sgd, prelude::*};
use gitcg_sim::{
    mcts::{CpuctConfig, MCTSConfig, MCTS},
    playout::Playout,
    rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng},
    training::{
        as_slice::*,
        features::{Features, InputFeatures},
        policy::*,
    },
};
use serde::Serialize;
use serde_json::json;
use structopt::StructOpt;

use ndarray::{Array1, Array2};

use gitcg_sim::{
    deck::cli_args::SearchOpts,
    game_tree_search::*,
    prelude::*,
    training::eval::*,
    types::{by_player::ByPlayer, nondet::NondetState},
};

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
    #[structopt(default_value = "4", long = "--min-depth")]
    pub mcts_min_depth: u8,
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
    #[structopt(help = "Run temporal different learning")]
    TDL {
        #[structopt(flatten)]
        search: SearchOpts,
        #[structopt(flatten)]
        tdl: TDLOpts,
    },
    #[structopt(help = "Run policy learning")]
    Policy {
        #[structopt(flatten)]
        search: SearchOpts,
        #[structopt(flatten)]
        policy: PolicyOpts,
    },
}

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

const N: usize = gitcg_sim::training::policy::N;
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

fn run_self_play<
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
    let evals = [PlayerId::PlayerFirst, PlayerId::PlayerSecond]
        .iter()
        .copied()
        .map(|player_id| {
            let eval_game_state = |game_state: &GameStateWrapper<S>| {
                if let Some(winner) = game_state.winner() {
                    winner_eval(winner, player_id)
                } else {
                    searches.get(player_id).get_self_play_model().evaluate(game_state)
                }
            };
            states.iter().map(eval_game_state).collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let evals = ByPlayer::new(evals[0].clone(), evals[1].clone());

    if let Some(make_debug_entry) = make_debug_entry {
        for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
            let model = searches[player_id].get_self_play_model();
            let entry = make_debug_entry(player_id, model.weights);
            println!("{}", serde_json::to_string(&entry).unwrap_or_default());
        }
    }

    for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
        let model = searches[player_id].get_self_play_model_mut();
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
    let searches = RefCell::new(ByPlayer::new(
        new_search(Default::default(), PlayerId::PlayerFirst, beta, delta),
        new_search(Default::default(), PlayerId::PlayerSecond, beta, delta),
    ));
    for i in 0..=opts.max_iters {
        deck.seed = Some(seed_gen.gen());
        let initial = deck.get_standard_game(Some(SmallRng::from_seed(seed_gen.gen())))?;
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
        run_self_play(
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

fn generate_batch<S: NondetState>(
    initial: &GameStateWrapper<S>,
    data_points: &mut Vec<(Features, InputFeatures<f32>, u8)>,
    searches: &RefCell<ByPlayer<MCTS<GameStateWrapper<S>>>>,
    mcts_min_depth: u8,
    batch_size: usize,
) -> bool {
    let mut batch_generated = false;
    while data_points.len() < batch_size {
        run_playout(initial, searches, 300, |searches, _game_state, _input| {
            let player_id = PlayerId::PlayerFirst;
            let mcts: &MCTS<GameStateWrapper<S>> = searches.get(player_id);
            let mut vec = vec![];
            mcts.get_self_play_data_points(player_id, mcts_min_depth, &mut vec);
            for (gs, input, depth) in vec {
                if input.player() != Some(PlayerId::PlayerFirst) {
                    continue;
                }
                let features = gs.features();
                let input_features = input.features(1f32);
                if data_points.len() >= batch_size {
                    return ControlFlow::Break(());
                }
                data_points.push((features, input_features, depth));
            }
            ControlFlow::Continue(())
        });
        batch_generated = true;
    }
    batch_generated
}

fn main_policy(deck: SearchOpts, opts: PolicyOpts) -> Result<(), std::io::Error> {
    let mut seed_gen = thread_rng();
    const BATCH_SIZE: usize = 512;
    let config = MCTSConfig {
        cpuct: CpuctConfig::STANDARD,
        random_playout_iters: 10,
        random_playout_bias: Some(50.0),
        random_playout_cutoff: 20,
        policy_bias: None,
        tt_size_mb: 32,
        limits: Some(SearchLimits {
            max_time_ms: Some(opts.mcts_time_limit_ms as u128),
            max_positions: None,
        }),
        debug: false,
        parallel: true,
    };

    let make_search = move || MCTS::new(config);

    let mut model: PolicyNetwork = PolicyNetwork::default();
    if let Some(npz_path) = &opts.load_npz {
        if let Err(e) = model.load_from_npz(npz_path) {
            println!(
                "{}",
                json!({
                    "load_error": format!("{e}")
                })
            );
        };
    }
    let mut grads = model.alloc_grads();
    let mut opt = Sgd::new(
        &model.model,
        SgdConfig {
            lr: 1e-1,
            weight_decay: opts.l2_regularization.map(WeightDecay::L2),
            ..Default::default()
        },
    );
    let mut data_points: Vec<(Features, InputFeatures<f32>, u8)> = vec![];
    for iter in 0..250 {
        let searches = RefCell::new(ByPlayer::new(make_search(), make_search()));
        let initial = deck.get_standard_game(Some(SmallRng::from_seed(seed_gen.gen())))?;
        let batch_generated = generate_batch(&initial, &mut data_points, &searches, opts.mcts_min_depth, BATCH_SIZE);
        let batch = (0..BATCH_SIZE).map(|_| data_points.pop().unwrap()).collect::<Vec<_>>();
        let (inputs, outputs): (Vec<_>, Vec<_>) = batch
            .iter()
            .take(BATCH_SIZE)
            .map(|(x, y, _)| (x.as_slice(), y.as_slice()))
            .unzip();
        let (inputs, outputs) = (inputs.concat(), outputs.concat());

        fn t_batched<const L: usize>(inputs: Vec<f32>) -> Array1<f32> {
            Array2::from_shape_vec((L, BATCH_SIZE), inputs)
                .unwrap()
                .t()
                .to_owned()
                .into_shape((BATCH_SIZE * L,))
                .unwrap()
        }

        let input_data = t_batched::<N>(inputs);
        let output_data = t_batched::<K>(outputs);

        let dev = &model.dev;
        let x = {
            let mut x = model.alloc_x::<BATCH_SIZE>();
            x.copy_from(input_data.as_slice().unwrap());
            x
        };
        let y = model.model.forward_mut(x.traced(grads));
        let mut y0: Tensor<Rank2<BATCH_SIZE, K>, f32, _> = dev.zeros();
        y0.copy_from(output_data.as_slice().unwrap());
        let loss = (y - y0).square().mean();
        let loss_value = loss.array();
        grads = loss.backward();
        opt.update(&mut model.model, &grads).unwrap();
        println!(
            "{}",
            json!({
                "loss": loss_value,
                "iter": iter,
                "batch_generated": batch_generated,
            })
        );
        if iter % 5 == 0 {
            if let Some(npz_path) = &opts.save_npz {
                model.save_npz(npz_path).unwrap();
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let opts = SelfPlayOpts::from_args();
    match opts {
        SelfPlayOpts::TDL { search: deck, tdl } => main_tdl(deck, tdl),
        SelfPlayOpts::Policy { search: deck, policy } => main_policy(deck, policy),
    }
}
