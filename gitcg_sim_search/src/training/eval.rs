use std::ops::Neg;

use crate::{
    mcts::{policy::EvalPolicy, MCTS},
    minimax::Eval,
    training::{as_slice::AsSlice, features::Features},
};
use gitcg_sim::{
    game_tree_search::*,
    linked_list,
    prelude::*,
    rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng},
};

use super::features::game_state_features;

/// Evaluates sigmoid(a * dot(x, w)) and its gradients over `x`.
pub fn sigmoid_dot<const LEN: usize>(x: &[f32; LEN], w: &[f32; LEN], a: f32) -> (f32, [f32; LEN]) {
    // Let $f(x; w) = sigmoid(a * (x.w))$
    // $df/dx_i = (d[sigmoid(a * (x.w))] / d[a * (x.w)]) (d[a * (x.w)] / dx_i) = a * sigmoid'(x . w) * w_i$
    let dot: f32 = a * x
        .iter()
        .copied()
        .zip(w.iter().copied())
        .fold(0f32, |s, (xi, wi)| s + xi * wi);
    let exp = dot.neg().exp();
    let y = 1f32 / (1f32 + exp);
    let exp1 = exp + 1f32;
    let dy = exp / (exp1 * exp1);
    let mut grads = [0f32; LEN];
    for (v, wi) in grads.iter_mut().zip(w.iter().copied()) {
        *v = a * dy * wi;
    }
    (y, grads)
}

#[derive(Default, Clone)]
pub struct SelfPlayModel {
    pub weights: Features,
    pub player_id: PlayerId,
}

impl SelfPlayModel {
    pub const LOSE: f32 = 0.0;
    pub const WIN: f32 = 1.0;
    // Modify this variable until eval per unit of HP is around 10
    pub const EVAL_SCALING: f32 = 100.0 / 1.25;
    const MULT: f32 = 1f32 / Self::EVAL_SCALING;

    pub fn new(weights: Features, player_id: PlayerId) -> Self {
        Self { weights, player_id }
    }

    pub fn evaluate<S: NondetState>(&self, game_state: &GameStateWrapper<S>) -> (f32, Features) {
        let features = game_state_features::features(&game_state.game_state);
        let x = <Features as AsSlice<f32>>::as_slice(features);
        let w = <Features as AsSlice<f32>>::as_slice_ref(&self.weights);
        let (y, grad) = sigmoid_dot(w, &x, Self::MULT);
        (y, <Features as AsSlice<f32>>::from_slice(grad))
    }
}

impl<S: NondetState> EvalPolicy<GameStateWrapper<S>> for SelfPlayModel {
    fn evaluate(&self, state: &GameStateWrapper<S>, player_id: PlayerId) -> <GameStateWrapper<S> as Game>::Eval {
        let v = SelfPlayModel::evaluate(self, state).0;
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
    pub fn new(init_weights: Features, player_id: PlayerId, beta: f32, delta: f32) -> Self {
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
                let eval = model.evaluate(&gs).0;
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
