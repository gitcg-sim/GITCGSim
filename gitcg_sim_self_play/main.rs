use std::fmt::Debug;

use gitcg_sim::rand::{rngs::SmallRng, thread_rng, Rng, SeedableRng};
// use gitcg_sim::mcts::{MCTSConfig, MCTS}
use itertools::Itertools;
use structopt::StructOpt;

use ndarray::Array1;

use gitcg_sim::{
    deck::cli_args::DeckOpts,
    game_tree_search::*,
    prelude::*,
    types::{by_player::ByPlayer, nondet::NondetState},
};

pub mod model;
use model::*;

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Genius Invokation TCG simulator - temporal difference learning")]
pub struct SelfPlayOpts {
    #[structopt(flatten)]
    pub deck: DeckOpts,
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

struct CSVDebug<'a, F: Fn(PlayerId) -> String> {
    pub index: i32,
    pub header_prefix: &'a str,
    pub row_prefix: F,
}

fn run_self_play<T: GameTreeSearch<GameStateWrapper<S>> + GetSelfPlayModel, S: NondetState>(
    initial: &GameStateWrapper<S>,
    searches: &mut ByPlayer<T>,
    learning_rate: f32,
    temporal_rate: f32,
    debug: Option<CSVDebug<impl Fn(PlayerId) -> String>>,
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
                        (
                            if winner == player_id {
                                SelfPlayModel::WIN
                            } else {
                                SelfPlayModel::LOSE
                            },
                            None,
                        )
                    } else {
                        searches.get(player_id).get_self_play_model().evaluate(game_state, true)
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let evals = ByPlayer::new(evals[0].clone(), evals[1].clone());

    if let Some(debug) = debug {
        if debug.index == 0 {
            let headers = Array1::from(states[0].features_headers());
            println!("{}, {}", debug.header_prefix, headers.into_iter().join(", "));
        } else {
            for player_id in [PlayerId::PlayerFirst, PlayerId::PlayerSecond] {
                let model = searches[player_id].get_self_play_model();
                let weights = model.weights.as_slice().unwrap();
                println!(
                    "{}, {}",
                    (debug.row_prefix)(player_id),
                    weights.iter().map(|x| format!("{:.8}", x)).join(", ")
                );
            }
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
                .into_iter()
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

fn main() -> Result<(), std::io::Error> {
    let mut seed_gen = thread_rng();
    let mut opts = SelfPlayOpts::from_args();
    let mut initial = opts.deck.get_standard_game(Some(SmallRng::from_seed(seed_gen.gen())))?;
    let (beta, delta) = (opts.beta, 1e-4);
    let n = initial.features_len();
    let mut searches = ByPlayer::new(
        new_search(Array1::zeros(n), PlayerId::PlayerFirst, beta, delta),
        new_search(Array1::zeros(n), PlayerId::PlayerSecond, beta, delta),
    );
    for i in 0..50_000i32 {
        opts.deck.seed = Some(seed_gen.gen());
        initial = opts.deck.get_standard_game(Some(SmallRng::from_seed(seed_gen.gen())))?;
        let learning_rate = opts.base_learning_rate * 0.95f32.powi(i / opts.learning_rate_decay);
        let debug = if i % 1000 == 0 {
            Some(CSVDebug {
                index: i,
                header_prefix: "iter, learning_rate, player_id",
                row_prefix: |player_id| format!("{i}, {learning_rate}, {player_id}"),
            })
        } else {
            None
        };
        run_self_play(&initial, &mut searches, learning_rate, opts.temporal_rate, debug);
    }
    Ok(())
}

// Example weights
// Scaling = 50.0, Func = Sigmoid: 1/(1+exp(-x))
// [DiceCount, HandCount, HP1,      HP2,       HP3,        ...]
// [3.4637372, 1.6737967, 9.720985, 11.761544, 11.649802, -3.455161, -1.6909992, -9.88536, -11.7617235, -11.677683], shape=[10], strides=[1], layout=CFcf (0xf), const ndim=1

// [DiceCount, HandCount, TeamStatusCount, SupportCount, HP1,      Energy1,  EquipCount1, StatusCount1,  AppliedCount1 ...]
// [4.0515056, 2.2079182, 4.771029,        0.771916,     8.263139, 3.609968, 2.789354,      2.6406562,   -1.2930163,   11.0483055, 2.9116597, 2.789354, 2.6406562, -2.5112936, 11.050083, 2.9868865, 2.789354, 2.6406562, -1.8632847, -4.051341, -2.2481544, -5.64001, -5.6321597, -8.598182, -2.6582355, -0.67318016, -2.8170075, 0.3311558, -11.066774, -2.8941412, -0.67318016, -2.8170075, 2.4254704, -11.093114, -2.8895025, -0.67318016, -2.8170075, 2.3620028]
