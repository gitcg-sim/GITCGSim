mod generator;
use generator::*;

use proptest::prelude::*;

use crate::{cards::ids::*, list8, prelude::*};

pub(crate) const CASES: u32 = 5_000;
// pub(crate) const CASES: u32 = 50_000;
// pub(crate) const CASES: u32 = 500_000;
// pub(crate) const CASES: u32 = 5_000_000;

pub mod state_evolution;

pub mod transpose;

pub mod zobrist_hash;

#[cfg(feature = "serde")]
pub mod serialization;
