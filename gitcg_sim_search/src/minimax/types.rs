use std::{cmp::min, ops::Neg};

use crate::*;

/// Heuristic value
pub type HV = i16;

/// A game state evaluation, which is based on number of moves until win/lose, and heuristic score
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eval {
    pub _repr: HV,
}

pub const WINNER: HV = 0x2000;
pub const THRESHOLD: HV = 0x500;
const UNIT_SIZES: [HV; 4] = [11, 42, 60, 120];
const DECAY: f32 = 0.95f32;

impl Eval {
    #[inline]
    const fn from_repr(eval: HV) -> Self {
        Self { _repr: eval }
    }

    #[inline]
    pub fn from_eval(eval: HV) -> Self {
        Self { _repr: eval }
    }

    #[inline]
    pub fn get_eval(self) -> HV {
        self._repr
    }

    #[inline]
    pub fn win(eval: HV) -> Self {
        Self { _repr: WINNER + eval }
    }

    #[inline]
    pub fn lose(eval: HV) -> Self {
        Self { _repr: -WINNER + eval }
    }
}

impl Neg for Eval {
    type Output = Self;

    #[inline]
    fn neg(self) -> Eval {
        Eval::from_repr(-self._repr)
    }
}

impl Windowable for Eval {
    #[inline]
    fn aspiration_window(self) -> (Self, Self) {
        let v = self._repr;
        (Self::from_repr(v - UNIT_SIZES[0]), Self::from_repr(v + UNIT_SIZES[0]))
    }

    #[inline]
    fn plus_unit(self, step: u8) -> Self {
        Self::from_repr(self._repr + UNIT_SIZES[min(step as usize, UNIT_SIZES.len() - 1)])
    }

    #[inline]
    fn minus_unit(self, step: u8) -> Self {
        Self::from_repr(self._repr - UNIT_SIZES[min(step as usize, UNIT_SIZES.len() - 1)])
    }

    #[inline]
    fn null_window(self) -> (Self, Self) {
        if self._repr >= THRESHOLD {
            (self, self.plus_one_step().plus_one_step())
        } else if self._repr <= THRESHOLD {
            (-self.plus_one_step().plus_one_step(), self)
        } else {
            (Self::from_repr(self._repr), Self::from_repr(self._repr + 1))
        }
    }
}

impl ValueTrait for Eval {}
impl EvalTrait for Eval {
    const MIN: Eval = Eval::from_repr(-WINNER);
    const MAX: Eval = Eval::from_repr(WINNER);

    #[inline]
    fn plus_one_step(self) -> Eval {
        if self._repr.abs() >= THRESHOLD {
            Self::from_repr((((self._repr as f32) * DECAY) as HV) - self._repr.signum())
        } else {
            self
        }
    }
}

impl std::fmt::Debug for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self._repr;
        write!(f, "Ev({:+3}.{:1})", v / 10, (v % 10).abs())
    }
}
