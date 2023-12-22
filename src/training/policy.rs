#[cfg(feature = "training")]
use dfdx::prelude::*;
#[cfg(feature = "training")]
use std::path::PathBuf;

use crate::{
    game_tree_search::Game,
    mcts::policy::*,
    prelude::GameStateWrapper,
    training::{
        as_slice::*,
        features::{Features, InputFeatures},
    },
    types::{input::Input, nondet::NondetState},
};

// const H: usize = 3;
// type Model = (Linear<N, H>, Sigmoid, Linear<H, K>, Sigmoid);
pub const N: usize = <Features as AsSlice<f32>>::LENGTH;
pub const K: usize = <InputFeatures<f32> as AsSlice<f32>>::LENGTH;

#[cfg(feature = "training")]
pub type Model = (Linear<N, K>, Sigmoid);

#[derive(Debug, Clone)]
pub struct PolicyNetwork {
    /// Use hard-coded model
    pub(crate) hard_coded: bool,
    #[cfg(feature = "training")]
    pub dev: Cpu,
    #[cfg(feature = "training")]
    pub model: <Model as dfdx::nn::BuildOnDevice<Cpu, f32>>::Built,
}

#[derive(Debug, Copy, Clone)]
pub struct TensorWrapper<T: Copy>(T);

impl<T: Copy> TensorWrapper<T> {
    pub fn array(&self) -> T {
        self.0
    }
}

#[cfg(not(feature = "training"))]
impl PolicyNetwork {
    pub fn new_hard_coded() -> Self {
        Self { hard_coded: true }
    }

    pub fn new() -> Self {
        Self::new_hard_coded()
    }

    pub fn eval(&self, x_slice: &[f32; N]) -> TensorWrapper<[f32; K]> {
        TensorWrapper(evaluate_hard_coded_policy(x_slice))
    }
}

#[cfg(feature = "training")]
impl PolicyNetwork {
    pub fn new() -> Self {
        let dev = Cpu::default();
        Self {
            hard_coded: false,
            dev: dev.clone(),
            model: dev.build_module::<Model, f32>(),
        }
    }

    pub fn new_hard_coded() -> Self {
        let dev = Cpu::default();
        Self {
            hard_coded: true,
            dev: dev.clone(),
            model: dev.build_module::<Model, f32>(),
        }
    }

    #[cfg(test)]
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

#[cfg(feature = "training")]
type ActionFeatures = Tensor1D<K>;
#[cfg(not(feature = "training"))]
type ActionFeatures = TensorWrapper<[f32; K]>;

type Action = Input;

pub struct SelectionPolicyState {
    pub puct_mult: f32,
    pub evals: smallvec::SmallVec<[f32; 16]>,
    pub denominator: f32,
}

impl PolicyNetwork {
    fn features_slice(action: Action) -> [f32; K] {
        action.features(1f32).as_slice()
    }

    pub(crate) fn action_value_hard_coded(action: Action, y: &[f32; K]) -> f32 {
        let w = Self::features_slice(action);
        let ww: f32 = w.iter().map(|x| x * x).sum();
        let yy: f32 = y.iter().map(|x| x * x).sum();
        let inner: f32 = w.iter().zip(y).map(|(wi, yi)| wi * yi).sum();
        inner / (ww * yy).sqrt()
    }

    #[cfg(feature = "training")]
    fn action_value_tensor(&self, action: Action, y: &ActionFeatures) -> f32 {
        let mut w: Tensor<Rank1<K>, f32, Cpu> = self.dev.zeros();
        w.copy_from(&Self::features_slice(action));
        let (ww, yy) = (w.clone().square().sum().array(), y.clone().square().sum().array());
        let w1: Tensor<Rank1<K>, f32, _> = w.reshape();
        let y1: Tensor<Rank2<K, 1>, f32, _> = y.clone().reshape();
        let inner: Tensor<Rank1<1>, f32, _> = w1.matmul(y1);
        let inner_f = inner.sum().array();
        inner_f / (ww * yy).sqrt()
    }

    pub(crate) fn action_value(&self, action: Action, y: &ActionFeatures) -> f32 {
        #[cfg(not(feature = "training"))]
        {
            Self::action_value_hard_coded(action, &y.array())
        }

        #[cfg(feature = "training")]
        {
            if self.hard_coded {
                Self::action_value_hard_coded(action, &y.array())
            } else {
                self.action_value_tensor(action, y)
            }
        }
    }
}

impl<S: NondetState> SelectionPolicy<GameStateWrapper<S>> for PolicyNetwork {
    type State = SelectionPolicyState;

    fn on_parent<F: FnOnce() -> <GameStateWrapper<S> as Game>::Actions>(
        &self,
        ctx: &SelectionPolicyContext<GameStateWrapper<S>>,
        get_children: F,
    ) -> Self::State {
        let parent = ctx.parent;
        let mut gs = parent.state.game_state.clone();
        if !ctx.is_maximize {
            gs.transpose_in_place();
        }
        let y = self.eval(&gs.express_features().as_slice());
        let mut denominator = 1e-5;
        let evals = get_children()
            .iter()
            .map(|&action| {
                let eval = self.action_value(action, &y);
                let v = ctx.config.policy_softmax(eval);
                denominator += v;
                v
            })
            .collect();
        let children_visits = (parent.prop.n - 1).max(1);
        SelectionPolicyState {
            puct_mult: Self::cpuct(ctx, parent.prop.n) * f32::sqrt(children_visits as f32),
            evals,
            denominator,
        }
    }

    fn policy(
        &self,
        _: &SelectionPolicyContext<GameStateWrapper<S>>,
        cctx: &SelectionPolicyChildContext<GameStateWrapper<S>, Self::State>,
    ) -> f32 {
        cctx.state.evals[cctx.index] / cctx.state.denominator
    }

    fn uct_child(
        &self,
        ctx: &SelectionPolicyContext<GameStateWrapper<S>>,
        cctx: &SelectionPolicyChildContext<GameStateWrapper<S>, Self::State>,
        policy_value: f32,
    ) -> f32 {
        let state = &cctx.state;
        let puct_mult = state.puct_mult;
        let n = cctx.child.prop.n;
        let n_child = (n + 1) as f32;
        let fpu = if n <= 10 * ctx.config.random_playout_iters {
            let fr = (n as f32) / ((10 * ctx.config.random_playout_iters) as f32);
            0.5 * (1.0 - fr) + 0.5
        } else {
            0.0
        };
        policy_value * (puct_mult / n_child) + fpu
    }
}

pub fn evaluate_hard_coded_policy(input: &[f32; N]) -> [f32; K] {
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

#[cfg(feature = "training")]
#[cfg(test)]
mod make_hard_coded_model {
    use super::*;

    mod requires_model_file {
        use super::*;
        const MODEL_PATH: &str = "./gitcg_sim_self_play/model_t3.npz";

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
        let y_eval = evaluate_hard_coded_policy(&input);
        assert_vectors_eq(&y_hard_coded, &y_eval);
    }
}
