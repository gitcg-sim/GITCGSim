use dfdx::prelude::*;
use std::path::PathBuf;

use crate::{
    mcts::{policy::*, Node},
    prelude::GameStateWrapper,
    training::{
        as_slice::*,
        features::{Features, InputFeatures},
    },
    types::nondet::NondetState,
};

// const H: usize = 3;
// type Model = (Linear<N, H>, Sigmoid, Linear<H, K>, Sigmoid);
pub const N: usize = <Features as AsSlice<f32>>::LENGTH;
pub const K: usize = <InputFeatures<f32> as AsSlice<f32>>::LENGTH;
pub type Model = (Linear<N, K>, Sigmoid);

#[derive(Debug, Clone)]
pub struct PolicyNetwork {
    pub dev: Cpu,
    pub model: <Model as dfdx::nn::BuildOnDevice<Cpu, f32>>::Built,
}

impl PolicyNetwork {
    pub fn new() -> Self {
        let dev = Cpu::default();
        Self {
            dev: dev.clone(),
            model: dev.build_module::<Model, f32>(),
        }
    }

    pub fn from_npz(path: &PathBuf) -> Result<Self, String> {
        let mut model = Self::new();
        model.load_from_npz(path)?;
        Ok(model)
    }

    pub fn alloc_grads(&self) -> Gradients<f32, Cpu> {
        self.model.alloc_grads()
    }

    pub fn alloc_x<const BATCH_SIZE: usize>(&self) -> Tensor<Rank2<BATCH_SIZE, N>, f32, Cpu> {
        self.dev.zeros()
    }

    pub fn alloc_y<const BATCH_SIZE: usize>(&self) -> Tensor<Rank2<BATCH_SIZE, K>, f32, Cpu> {
        self.dev.zeros()
    }

    pub fn load_from_npz(&mut self, path: &PathBuf) -> Result<(), String> {
        self.model.load(path).map_err(|e| e.to_string())
    }

    pub fn save_npz(&mut self, path: &PathBuf) -> Result<(), String> {
        self.model.save(path).map_err(|e| e.to_string())
    }
}

impl Default for PolicyNetwork {
    fn default() -> Self {
        Self::new()
    }
}

// pub struct SelectionPolicyState {
//     pub mult: f32,
//     pub y: Tensor<Rank1<K>, f32, Cpu>,
//     pub move_evals: Vec<Tensor<Rank1<K>, f32, Cpu>>,
//     // TODO continue
// }

impl<S: NondetState> SelectionPolicy<GameStateWrapper<S>> for PolicyNetwork {
    type State = (f32, Tensor1D<K>);

    fn uct_parent_factor(&self, ctx: &SelectionPolicyContext<GameStateWrapper<S>>) -> Self::State {
        let parent = ctx.parent;
        let n_parent = parent.prop.n;
        let model = &self.model;
        let mut x: Tensor<Rank1<N>, f32, Cpu> = self.dev.zeros();
        let mut gs = parent.state.game_state.clone();
        if !ctx.is_maximize {
            gs.transpose_in_place();
        }
        let slice = gs.express_features().as_slice();
        x.copy_from(&slice);
        let y = model.forward(x);
        (ctx.config.c * (n_parent as f32).sqrt(), y)
    }

    fn uct_child_factor(
        &self,
        ctx: &SelectionPolicyContext<GameStateWrapper<S>>,
        child: &Node<GameStateWrapper<S>>,
        (f, y): &Self::State,
    ) -> f32 {
        let mut w: Tensor<Rank1<K>, f32, Cpu> = self.dev.zeros();
        if let Some(a) = child.action {
            w.copy_from(&a.features(1f32).as_slice());
        }
        let ww = w.clone().square().sum().array();
        let mm = y.clone().square().sum().array();
        w.axpy(1, y, 1);
        let v = w.sum().array() / (ww * mm).sqrt();

        let a = if let Some(a) = ctx.config.random_playout_bias {
            (v * a).exp().clamp(1e-2, 1e2)
        } else {
            v
        };
        let n_child = child.prop.n + 1;
        (a * f / (n_child as f32)).sqrt()
    }
}
