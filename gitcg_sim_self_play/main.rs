use std::fmt::Debug;

use itertools::Itertools;
use gitcg_sim::rand::{distributions::WeightedIndex, prelude::Distribution, rngs::SmallRng, thread_rng, Rng, SeedableRng};
use structopt::StructOpt;

use ndarray::{Array1, Dim};
use neuronika::{Var, VarDiff};

use gitcg_sim::{
    deck::cli_args::DeckOpts,
    game_tree_search::*,
    linked_list,
    minimax::Eval,
    prelude::*,
    types::{by_player::ByPlayer, nondet::NondetState},
};

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

#[derive(Default, Clone)]
struct SelfPlayModel {
    pub weights: Array1<f32>,
}

impl SelfPlayModel {
    pub const LOSE: f32 = 0.0;
    pub const WIN: f32 = 1.0;
    // Modify this variable until eval per unit of HP is around 10
    pub const EVAL_SCALING: f32 = 50.0;

    fn network(x: Var<Dim<[usize; 1]>>, w: VarDiff<Dim<[usize; 1]>>) -> VarDiff<Dim<[usize; 0]>> {
        (w.vv(x) / Self::EVAL_SCALING).sigmoid()
    }

    #[allow(dead_code)]
    pub fn new(weights: Array1<f32>) -> Self {
        Self { weights }
    }

    fn evaluate<S: NondetState>(&self, game_state: &GameStateWrapper<S>, grad: bool) -> (f32, Option<Array1<f32>>) {
        let features = game_state.features();
        let x = neuronika::from_ndarray(Array1::from_vec(features));
        let w = neuronika::from_ndarray(self.weights.clone()).requires_grad();
        let y = Self::network(x, w.clone());
        y.forward();
        let yv = (&y.data())[()];
        if grad {
            y.backward(1.0);
            let grad = w.grad().to_owned();
            (yv, Some(grad))
        } else {
            (yv, None)
        }
    }
}

/// Boltzmann action selection
/// Select action based on weight: exp(x * beta) + random(0..delta)
/// where x is the evaluation of the position after making a particular action.
struct SelfPlaySearch {
    pub model: SelfPlayModel,
    pub beta: f32,
    pub delta: f32,
}

impl SelfPlaySearch {
    pub fn new<S: NondetState>(s: &GameStateWrapper<S>, beta: f32, delta: f32) -> Self {
        let n = s.features_len();
        let weights = Array1::zeros(n);
        Self {
            model: SelfPlayModel::new(weights),
            beta,
            delta,
        }
    }
}

impl<S: NondetState> GameTreeSearch<GameStateWrapper<S>> for SelfPlaySearch {
    fn search(&mut self, position: &GameStateWrapper<S>, _: PlayerId) -> SearchResult<GameStateWrapper<S>> {
        let model = &self.model;
        let actions = position.actions();
        let pairs = actions
            .iter()
            .map(|&a| {
                let mut gs = position.clone();
                gs.advance(a).unwrap();
                let eval = model.evaluate(&gs, false).0;
                (
                    (eval * self.beta).exp() + thread_rng().gen_range(0f32..self.delta),
                    eval,
                )
            })
            .collect::<Vec<_>>();
        let weights = pairs.iter().map(|x| x.0).collect::<Vec<_>>();
        let mut rng = thread_rng();
        let Ok(dist) = WeightedIndex::new(weights) else {
            panic!("SelfPlaySearch: Invalid weights: {:?}", &pairs);
        };
        let i_best = dist.sample(&mut rng);
        let (selected, best_eval) = (actions[i_best], pairs[i_best].1);
        SearchResult {
            pv: linked_list![selected],
            eval: Eval::from_eval((best_eval * 1e3) as i16),
            counter: Default::default(),
        }
    }
}

fn run_playout<T: GameTreeSearch<GameStateWrapper<S>>, S: NondetState>(
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

fn run_self_play<S: NondetState>(
    initial: &GameStateWrapper<S>,
    models: &mut ByPlayer<SelfPlaySearch>,
    learning_rate: f32,
    temporal_rate: f32,
    debug: Option<CSVDebug<impl Fn(PlayerId) -> String>>,
) {
    let states = run_playout(initial, models, 300).unwrap();
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
                        models.get(player_id).model.evaluate(game_state, true)
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
                let model = &models[player_id].model;
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
        let model = &mut models[player_id].model;
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

fn main() -> Result<(), std::io::Error> {
    let mut seed_gen = thread_rng();
    let mut opts = SelfPlayOpts::from_args();
    let mut initial = opts.deck.get_standard_game(Some(SmallRng::from_seed(seed_gen.gen())))?;
    let (beta, delta) = (opts.beta, 1e-4);
    let mut models = ByPlayer::new(
        SelfPlaySearch::new(&initial, beta, delta),
        SelfPlaySearch::new(&initial, beta, delta),
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
        run_self_play(&initial, &mut models, learning_rate, opts.temporal_rate, debug);
    }
    Ok(())
}

// Example weights
// Scaling = 50.0, Func = Sigmoid: 1/(1+exp(-x))
// [DiceCount, HandCount, HP1,      HP2,       HP3,        ...]
// [3.4637372, 1.6737967, 9.720985, 11.761544, 11.649802, -3.455161, -1.6909992, -9.88536, -11.7617235, -11.677683], shape=[10], strides=[1], layout=CFcf (0xf), const ndim=1

// [DiceCount, HandCount, TeamStatusCount, SupportCount, HP1,      Energy1,  EquipCount1, StatusCount1,  AppliedCount1 ...]
// [4.0515056, 2.2079182, 4.771029,        0.771916,     8.263139, 3.609968, 2.789354,      2.6406562,   -1.2930163,   11.0483055, 2.9116597, 2.789354, 2.6406562, -2.5112936, 11.050083, 2.9868865, 2.789354, 2.6406562, -1.8632847, -4.051341, -2.2481544, -5.64001, -5.6321597, -8.598182, -2.6582355, -0.67318016, -2.8170075, 0.3311558, -11.066774, -2.8941412, -0.67318016, -2.8170075, 2.4254704, -11.093114, -2.8895025, -0.67318016, -2.8170075, 2.3620028]
