use dfdx::prelude::*;
use std::path::PathBuf;

use crate::training::{
    as_slice::*,
    features::{GameStateFeatures, InputFeatures},
};

// const H: usize = 3;
// type Model = (Linear<N, H>, Sigmoid, Linear<H, K>, Sigmoid);
pub const N: usize = <GameStateFeatures<f32> as AsSlice<f32>>::LENGTH;
pub const K: usize = <InputFeatures<f32> as AsSlice<f32>>::LENGTH;
pub type Model = (Linear<N, K>, Sigmoid);

#[derive(Debug, Clone)]
pub struct PolicyNetwork {
    pub dev: Cpu,
    pub model: <(Linear<N, K>, Sigmoid) as dfdx::nn::BuildOnDevice<Cpu, f32>>::Built,
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
