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

    pub fn load_hard_coded(&mut self) {
        let (lin, _) = &mut self.model;
        lin.weight.copy_from(&super::hard_coded_model::LIN_WEIGHT);
        lin.bias.copy_from(&super::hard_coded_model::LIN_BIAS);
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

    pub fn eval(&self, x_slice: &[f32; N]) -> Tensor<Rank1<K>, f32, Cpu> {
        let model = &self.model;
        let mut x = self.alloc_x::<1>();
        x.copy_from(x_slice);
        model.forward(x).reshape()
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
        (ctx.config.c * (n_parent as f32).ln_1p(), y)
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
            1e-2 + v
        };
        let n_child = child.prop.n + 1;
        f * (a / (n_child as f32)).sqrt()
    }
}

pub fn evaluate_hard_coded_policy(input: [f32; N]) -> [f32; K] {
    let mut y = [0f32; K];
    for (i, yi) in y.iter_mut().enumerate() {
        let mut s = super::hard_coded_model::LIN_BIAS[i];
        for (j, xj) in input.iter().copied().enumerate() {
            s += super::hard_coded_model::LIN_WEIGHT[i * N + j] * xj;
        }
        *yi = 1.0 / (1.0 + (-s).exp());
    }
    y
}

#[cfg(test)]
mod make_hard_coded_model {
    use super::*;

    mod requires_model_file {
        use super::*;
        const MODEL_PATH: &str = "./gitcg_sim_self_play/model_standard.npz";

        fn npz_path() -> PathBuf {
            PathBuf::from(std::ffi::OsStr::new(MODEL_PATH))
        }

        // Uncomment this method to generate
        #[test]
        fn gen_hard_coded_model() {
            let mut model = PolicyNetwork::new();
            model.load_from_npz(&npz_path()).unwrap();
            let lin = &model.model.0;
            let lin_weight = lin.weight.clone().reshape::<Rank1<{ N * K }>>().array();
            let lin_bias = lin.bias.array();
            println!("// Generated code, see ./policy.rs make_hard_coded_model::gen_hard_coded_model()");
            println!("pub const LIN_WEIGHT: [f32; {}] = {lin_weight:#?};", N * K);
            println!("pub const LIN_BIAS: [f32; {K}] = {lin_bias:#?};");
            println!("}}");
        }

        #[test]
        fn hard_coded_model_loaded_correctly() {
            let mut model_loaded = PolicyNetwork::new();
            model_loaded.load_from_npz(&npz_path()).unwrap();
            let input = rand_input(1);
            let mut model_hard_coded = PolicyNetwork::new();
            model_hard_coded.load_hard_coded();
            let y_hard_coded = model_hard_coded.eval(&input).array();
            let y_loaded = model_loaded.eval(&input).array();
            assert_vectors_eq(&y_loaded, &y_hard_coded);
        }
    }

    fn rand_input(seed: u64) -> [f32; N] {
        use rand::{rngs::SmallRng, Rng, SeedableRng};
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut input: [f32; N] = [0.0; N];
        for xi in input.iter_mut() {
            *xi = rng.gen_range(0f32..1f32);
        }
        input
    }

    fn assert_vectors_eq(expected: &[f32], actual: &[f32]) {
        let sum_sq_diff: f32 = itertools::Zip::from((expected, actual))
            .map(|(&a, &b)| (b - a) * (b - a))
            .sum();
        assert_eq!(expected.len(), actual.len());
        assert!(sum_sq_diff <= 1e-4);
    }

    #[test]
    fn hard_coded_model_calculated_correctly() {
        let mut model_hard_coded = PolicyNetwork::new();
        model_hard_coded.load_hard_coded();
        let input = rand_input(1);
        let y_hard_coded = model_hard_coded.eval(&input).array();
        let y_eval = evaluate_hard_coded_policy(input);
        assert_vectors_eq(&y_hard_coded, &y_eval);
    }
}
