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
pub struct PolicyOpts {}

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

fn run_playout<T: GameTreeSearch<GameStateWrapper<S>> + GetSelfPlayModel, S: NondetState>(
    initial: &GameStateWrapper<S>,
    models: &mut ByPlayer<T>,
    max_iters: usize,
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
    let states = run_playout(initial, searches, 300).unwrap();
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
            const N: usize = <GameStateFeatures<f32> as AsSlice>::LENGTH;
            if weights.len() != N {
                panic!();
            }
            let mut slice: <GameStateFeatures<f32> as AsSlice>::Slice = [0f32; N];
            slice[..weights.len()].copy_from_slice(weights);
            let entry = make_debug_entry(player_id, <GameStateFeatures<f32> as AsSlice>::from_slice(slice));
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

#[cfg(any())]
fn new_search<S: NondetState>(
    init_weights: Array1<f32>,
    player_id: PlayerId,
    beta: f32,
    delta: f32,
) -> MCTS<GameStateWrapper<S>, SelfPlayModel> {
    let model = SelfPlayModel::new(init_weights, player_id);
    let config = MCTSConfig {
        c: 2.0,
        b: None,
        tt_size_mb: 0,
        parallel: true,
        random_playout_iters: 100,
        random_playout_cutoff: 50,
        random_playout_bias: Some(0.5),
        debug: false,
        limits: Some(SearchLimits {
            max_time_ms: Some(100),
            max_positions: None,
        }),
    };
    MCTS::new_with_eval_policy(config, model)
}

fn main_tdl(mut deck: DeckOpts, opts: TDLOpts) -> Result<(), std::io::Error> {
    let mut seed_gen = thread_rng();
    let (beta, delta) = (opts.beta, 1e-4);
    const N: usize = <GameStateFeatures<f32> as AsSlice>::LENGTH;
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

fn main() -> Result<(), std::io::Error> {
    let opts = SelfPlayOpts::from_args();
    match opts {
        SelfPlayOpts::TDL { deck, tdl } => main_tdl(deck, tdl),
        SelfPlayOpts::Policy { deck: _, policy: _ } => todo!(),
    }
}

// Example weights
// Scaling = 50.0, Func = Sigmoid: 1/(1+exp(-x))
// [DiceCount, HandCount, HP1,      HP2,       HP3,        ...]
// [3.4637372, 1.6737967, 9.720985, 11.761544, 11.649802, -3.455161, -1.6909992, -9.88536, -11.7617235, -11.677683], shape=[10], strides=[1], layout=CFcf (0xf), const ndim=1

// [DiceCount, HandCount, TeamStatusCount, SupportCount, HP1,      Energy1,  EquipCount1, StatusCount1,  AppliedCount1 ...]
// [4.0515056, 2.2079182, 4.771029,        0.771916,     8.263139, 3.609968, 2.789354,      2.6406562,   -1.2930163,   11.0483055, 2.9116597, 2.789354, 2.6406562, -2.5112936, 11.050083, 2.9868865, 2.789354, 2.6406562, -1.8632847, -4.051341, -2.2481544, -5.64001, -5.6321597, -8.598182, -2.6582355, -0.67318016, -2.8170075, 0.3311558, -11.066774, -2.8941412, -0.67318016, -2.8170075, 2.4254704, -11.093114, -2.8895025, -0.67318016, -2.8170075, 2.3620028]
