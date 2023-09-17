use std::{fmt::Debug, ops::ControlFlow};

use dfdx::{optim::Sgd, prelude::*, tensor::*};
use gitcg_sim::{
    mcts::{MCTSConfig, MCTS},
    rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng},
    training::{
        as_slice::*,
        features::{GameStateFeatures, InputFeatures},
    },
};
use serde::Serialize;
use serde_json::json;
use structopt::StructOpt;

use ndarray::{Array1, Array2};

use gitcg_sim::{
    deck::cli_args::DeckOpts,
    game_tree_search::*,
    prelude::*,
    types::{by_player::ByPlayer, nondet::NondetState},
};

pub mod model;
use model::*;

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
}

#[derive(Debug, StructOpt, Clone)]
pub struct PolicyOpts {
    #[structopt(default_value = "250", long = "--mcts-time-limit-ms")]
    pub mcts_time_limit_ms: f32,
    #[structopt(default_value = "4", long = "--min-depth")]
    pub mcts_min_depth: u8,
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Genius Invokation TCG simulator - self-play")]
pub enum SelfPlayOpts {
    #[structopt(help = "Run temporal different learning")]
    TDL {
        #[structopt(flatten)]
        deck: DeckOpts,
        #[structopt(flatten)]
        tdl: TDLOpts,
    },
    #[structopt(help = "Run policy learning")]
    Policy {
        #[structopt(flatten)]
        deck: DeckOpts,
        #[structopt(flatten)]
        policy: PolicyOpts,
    },
}

fn run_playout<
    T: GameTreeSearch<GameStateWrapper<S>> + GetSelfPlayModel,
    S: NondetState,
    C: FnMut(&ByPlayer<T>, &GameStateWrapper<S>, Input) -> ControlFlow<()>,
>(
    initial: &GameStateWrapper<S>,
    models: &mut ByPlayer<T>,
    max_iters: usize,
    mut on_step: C,
) -> Result<Vec<GameStateWrapper<S>>, DispatchError> {
    let mut game_state = initial.clone();
    let mut states: Vec<GameStateWrapper<S>> = Vec::with_capacity(16);
    for _ in 0..max_iters {
        states.push(game_state.clone());
        if game_state.winner().is_some() {
            break;
        }
        let player_id = game_state.to_move().unwrap();
        let model = models.get_mut(player_id);
        let input = {
            let mut gs1 = game_state.clone();
            gs1.hide_private_information(player_id.opposite());
            let res = model.search(&gs1, player_id);
            res.pv.head().unwrap()
        };
        game_state.advance(input)?;
        match on_step(models, &game_state, input) {
            ControlFlow::Continue(_) => {}
            ControlFlow::Break(_) => break,
        }
    }
    Ok(states)
}

fn winner_eval(winner: PlayerId, player_id: PlayerId) -> (f32, Option<Array1<f32>>) {
    (
        if winner == player_id {
            SelfPlayModel::WIN
        } else {
            SelfPlayModel::LOSE
        },
        None,
    )
}

#[derive(Serialize)]
struct DebugEntry {
    pub iter: u32,
    pub learning_rate: f32,
    pub player_id: PlayerId,
    #[serde(flatten)]
    pub game_state: GameStateFeatures<f32>,
}

fn run_self_play<
    T: GameTreeSearch<GameStateWrapper<S>> + GetSelfPlayModel,
    S: NondetState,
    F: Fn(PlayerId, GameStateFeatures<f32>) -> DebugEntry,
>(
    initial: &GameStateWrapper<S>,
    searches: &mut ByPlayer<T>,
    learning_rate: f32,
    temporal_rate: f32,
    make_debug_entry: Option<F>,
) {
    let states = run_playout(initial, searches, 300, |_, _, _| ControlFlow::Continue(())).unwrap();
    let evals = [PlayerId::PlayerFirst, PlayerId::PlayerSecond]
        .iter()
        .copied()
        .map(|player_id| {
            states
                .iter()
                .map(|game_state| {
                    if let Some(winner) = game_state.winner() {
                        winner_eval(winner, player_id)
                    } else {
                        searches.get(player_id).get_self_play_model().evaluate(game_state, true)
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let evals = ByPlayer::new(evals[0].clone(), evals[1].clone());

    if let Some(make_debug_entry) = make_debug_entry {
        for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
            let model = searches[player_id].get_self_play_model();
            let weights = model.weights.as_slice().unwrap();
            const N: usize = <GameStateFeatures<f32> as AsSlice<f32>>::LENGTH;
            if weights.len() != N {
                panic!();
            }
            let mut slice: <GameStateFeatures<f32> as AsSlice<f32>>::Slice = [0f32; N];
            slice[..weights.len()].copy_from_slice(weights);
            let entry = make_debug_entry(player_id, <GameStateFeatures<f32> as AsSlice<f32>>::from_slice(slice));
            println!("{}", serde_json::to_string(&entry).unwrap_or_default());
        }
    }

    for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
        let model = searches[player_id].get_self_play_model_mut();
        let weights = &mut model.weights;
        let n_states = states.len();
        let evals = &evals[player_id];
        for t in 0..(n_states - 1) {
            let Some(grad) = &evals[t].1 else { continue };
            let sum_td: f32 = (t..(n_states - 1))
                .map(|j| {
                    let temporal_diff = evals[j + 1].0 - evals[j].0;
                    temporal_rate.powi((j - t) as i32) * temporal_diff
                })
                .sum();
            weights.scaled_add(sum_td * learning_rate, grad);
        }
    }
}

fn new_search(init_weights: Array1<f32>, player_id: PlayerId, beta: f32, delta: f32) -> SelfPlaySearch {
    SelfPlaySearch::new(init_weights, player_id, beta, delta)
}

fn main_tdl(mut deck: DeckOpts, opts: TDLOpts) -> Result<(), std::io::Error> {
    let mut seed_gen = thread_rng();
    let (beta, delta) = (opts.beta, 1e-4);
    const N: usize = <GameStateFeatures<f32> as AsSlice<f32>>::LENGTH;
    let mut searches = ByPlayer::new(
        new_search(Array1::zeros(N), PlayerId::PlayerFirst, beta, delta),
        new_search(Array1::zeros(N), PlayerId::PlayerSecond, beta, delta),
    );
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
        run_self_play(&initial, &mut searches, learning_rate, opts.temporal_rate, debug);
    }
    Ok(())
}

const H: usize = 8;
const N: usize = <GameStateFeatures<f32> as AsSlice<f32>>::LENGTH;
const K: usize = <InputFeatures<f32> as AsSlice<f32>>::LENGTH;
type Model = (Linear<N, H>, Sigmoid, Linear<H, K>, Sigmoid);

fn main_policy(deck: DeckOpts, opts: PolicyOpts) -> Result<(), std::io::Error> {
    let mut seed_gen = thread_rng();
    const BATCH_SIZE: usize = 512;
    let config = MCTSConfig {
        c: 2.0,
        b: None,
        random_playout_iters: 10,
        random_playout_bias: Some(50.0),
        random_playout_cutoff: 20,
        tt_size_mb: 32,
        limits: Some(SearchLimits {
            max_time_ms: Some(opts.mcts_time_limit_ms as u128),
            max_positions: None,
        }),
        debug: false,
        parallel: true,
    };

    let dev: Cpu = Default::default();
    let mut model = dev.build_module::<Model, f32>();
    let mut grads = model.alloc_grads();
    let mut opt = Sgd::new(&model, Default::default());
    for iter in 0..250 {
        let mut searches = ByPlayer::new(MCTS::new(config), MCTS::new(config));
        let initial = deck.get_standard_game(Some(SmallRng::from_seed(seed_gen.gen())))?;
        let mut data_points: Vec<(GameStateFeatures<f32>, InputFeatures<f32>, u8)> = vec![];
        while data_points.len() < BATCH_SIZE {
            run_playout(&initial, &mut searches, 300, |searches, _game_state, _input| {
                let player_id = PlayerId::PlayerFirst;
                let mcts = &searches[player_id];
                let mut vec = vec![];
                mcts.get_self_play_data_points(player_id, opts.mcts_min_depth, &mut vec);
                for (gs, input, depth) in vec {
                    if input.player() != Some(PlayerId::PlayerFirst) {
                        continue;
                    }
                    let features = gs.features();
                    let input_features = input.features(1f32);
                    #[cfg(any())]
                    println!(
                        "{{\"depth\": {depth}, \"game_state\": {}, \"input\": {}}}",
                        serde_json::to_string(&features).unwrap(),
                        serde_json::to_string(&input_features).unwrap()
                    );
                    if data_points.len() >= BATCH_SIZE {
                        return ControlFlow::Break(());
                    }
                    data_points.push((features, input_features, depth));
                }
                ControlFlow::Continue(())
            })
            .unwrap();
        }

        // let rows = data_points.len();
        let input_data = {
            let inputs = data_points
                .iter()
                .take(BATCH_SIZE)
                .flat_map(|(x, _, _)| x.as_slice())
                .collect::<Vec<f32>>();
            Array2::from_shape_vec((N, BATCH_SIZE), inputs).unwrap().t().to_owned()
        };
        let output_data = {
            let inputs = data_points
                .iter()
                .take(BATCH_SIZE)
                .flat_map(|(_, y, _)| y.as_slice())
                .collect::<Vec<f32>>();
            Array2::from_shape_vec((K, BATCH_SIZE), inputs).unwrap().t().to_owned()
        };

        let mut x: Tensor<Rank2<BATCH_SIZE, N>, f32, _> = dev.zeros();
        x.copy_from(input_data.into_shape((BATCH_SIZE * N,)).unwrap().as_slice().unwrap());
        let y = model.forward_mut(x.traced(grads));
        let mut y0: Tensor<Rank2<BATCH_SIZE, K>, f32, _> = dev.zeros();
        y0.copy_from(output_data.into_shape((BATCH_SIZE * K,)).unwrap().as_slice().unwrap());
        let loss = (y - y0).square().mean();
        let loss_value = loss.array();
        grads = loss.backward();
        opt.update(&mut model, &grads).unwrap();
        println!(
            "{}",
            json!({
                "loss": loss_value,
                "iter": iter
            })
        );
        #[cfg(any())]
        {
            let policy_matrix_struct = {
                let flattened: [f32; N * K] = unsafe { std::mem::transmute(model.0.weight.array()) };
                <InputFeatures<GameStateFeatures<f32>> as AsSlice>::from_slice(flattened)
            };
            let policy_bias_struct = <InputFeatures<f32> as AsSlice>::from_slice(model.0.bias.array());
            let log_entry = json!({
                "policy": {
                    "matrix": policy_matrix_struct,
                    "bias": policy_bias_struct
                },
                "iter": iter,
                "loss": loss_value
            });
            println!("{}", log_entry);
        }
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let opts = SelfPlayOpts::from_args();
    match opts {
        SelfPlayOpts::TDL { deck, tdl } => main_tdl(deck, tdl),
        SelfPlayOpts::Policy { deck, policy } => main_policy(deck, policy),
    }
}
