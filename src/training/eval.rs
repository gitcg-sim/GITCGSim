use crate::{
    game_tree_search::*,
    linked_list,
    mcts::{policy::EvalPolicy, MCTS},
    minimax::Eval,
    prelude::*,
    rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng},
    training::{as_slice::AsSlice, features::GameStateFeatures},
    types::nondet::NondetState,
};

use ndarray::{Array1, Dim};
use neuronika::{Var, VarDiff};

#[derive(Default, Clone)]
pub struct SelfPlayModel {
    pub weights: Array1<f32>,
    pub player_id: PlayerId,
}

impl SelfPlayModel {
    pub const LOSE: f32 = 0.0;
    pub const WIN: f32 = 1.0;
    // Modify this variable until eval per unit of HP is around 10
    pub const EVAL_SCALING: f32 = 100.0 / 1.25;

    pub fn network(x: Var<Dim<[usize; 1]>>, w: VarDiff<Dim<[usize; 1]>>) -> VarDiff<Dim<[usize; 0]>> {
        (w.vv(x) / Self::EVAL_SCALING).sigmoid()
    }

    pub fn new(weights: Array1<f32>, player_id: PlayerId) -> Self {
        Self { weights, player_id }
    }

    pub fn evaluate<S: NondetState>(&self, game_state: &GameStateWrapper<S>, grad: bool) -> (f32, Option<Array1<f32>>) {
        let features = game_state.features();
        let slice = <GameStateFeatures<f32> as AsSlice<f32>>::as_slice(features);
        let x = neuronika::from_ndarray(Array1::from_iter(slice));
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

impl<S: NondetState> EvalPolicy<GameStateWrapper<S>> for SelfPlayModel {
    fn evaluate(&self, state: &GameStateWrapper<S>, player_id: PlayerId) -> <GameStateWrapper<S> as Game>::Eval {
        let v = SelfPlayModel::evaluate(self, state, false).0;
        let v = if player_id == self.player_id { v } else { 1.0 - v };
        Eval::from_eval((v * 1e3) as i16)
    }
}

pub trait GetSelfPlayModel {
    fn get_self_play_model(&self) -> &SelfPlayModel;
    fn get_self_play_model_mut(&mut self) -> &mut SelfPlayModel;
}

/// Boltzmann action selection
/// Select action based on weight: exp(x * beta) + random(0..delta)
/// where x is the evaluation of the position after making a particular action.
pub struct SelfPlaySearch {
    pub model: SelfPlayModel,
    pub beta: f32,
    pub delta: f32,
}

impl GetSelfPlayModel for SelfPlaySearch {
    #[inline(always)]
    fn get_self_play_model(&self) -> &SelfPlayModel {
        &self.model
    }

    #[inline(always)]
    fn get_self_play_model_mut(&mut self) -> &mut SelfPlayModel {
        &mut self.model
    }
}

impl SelfPlaySearch {
    pub fn new(init_weights: Array1<f32>, player_id: PlayerId, beta: f32, delta: f32) -> Self {
        Self {
            model: SelfPlayModel::new(init_weights, player_id),
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

impl<S: NondetState> GetSelfPlayModel for MCTS<GameStateWrapper<S>, SelfPlayModel> {
    #[inline(always)]
    fn get_self_play_model(&self) -> &SelfPlayModel {
        &self.eval_policy
    }

    #[inline(always)]
    fn get_self_play_model_mut(&mut self) -> &mut SelfPlayModel {
        &mut self.eval_policy
    }
}
