use std::{cmp::min, ops::Neg};

use serde::{Deserialize, Serialize};

use super::*;

/// Heuristic value
pub type HV = i16;

pub const WINNER_UNIT: HV = 2048;
pub const MAX_WINNER_STEPS: HV = 8;
pub const MAX_WINNER_FOUND_VALUE: HV = MAX_WINNER_STEPS * WINNER_UNIT;

/// A game state evaluation, which is based on number of moves until win/lose, and heuristic score
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Eval {
    pub _repr: HV,
}

const UNIT_SIZES: [HV; 4] = [11, 42, 60, 120];

impl Eval {
    const fn from_repr_const(eval: HV) -> Self {
        Self { _repr: eval }
    }

    #[inline]
    fn from_repr(eval: HV) -> Self {
        Self { _repr: eval }
    }

    #[inline]
    pub fn from_eval(eval: HV) -> Self {
        if eval >= WINNER_UNIT {
            Self { _repr: WINNER_UNIT - 1 }
        } else if eval <= -WINNER_UNIT {
            Self {
                _repr: -WINNER_UNIT + 1,
            }
        } else {
            Self { _repr: eval }
        }
    }

    #[inline]
    pub fn get_eval(self) -> HV {
        self._repr
    }

    #[inline]
    pub fn from_winner_steps(winner_steps: i8) -> Self {
        if winner_steps == 0 {
            return Self::default();
        }
        let n = (winner_steps as HV)
            .max(-MAX_WINNER_STEPS + 1)
            .min(MAX_WINNER_STEPS - 1);
        let u = WINNER_UNIT * n.signum();
        Self::from_repr(MAX_WINNER_STEPS.saturating_sub(n.abs()) * u)
    }

    #[inline]
    pub fn get_winner_steps(self) -> i8 {
        if self.is_eval() {
            return 0;
        }
        let n = self._repr / WINNER_UNIT;
        (MAX_WINNER_STEPS.saturating_sub(n.abs()) * n.signum()) as i8
    }

    #[inline]
    pub fn is_eval(self) -> bool {
        let v = self._repr;
        v < WINNER_UNIT && v > -WINNER_UNIT
    }

    pub const fn win() -> Eval {
        Self::from_repr_const(MAX_WINNER_FOUND_VALUE - WINNER_UNIT)
    }

    #[inline]
    pub const fn lose() -> Eval {
        Self::from_repr_const(-MAX_WINNER_FOUND_VALUE + WINNER_UNIT)
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
        if !self.is_eval() {
            return (Self::MIN, Self::MAX);
        }

        let v = self._repr;
        (Self::from_repr(v - UNIT_SIZES[0]), Self::from_repr(v + UNIT_SIZES[0]))
    }

    #[inline]
    fn plus_unit(self, step: u8) -> Self {
        if self.is_eval() {
            Self::from_repr(self._repr + UNIT_SIZES[min(step as usize, UNIT_SIZES.len() - 1)])
        } else {
            self.plus_one_step()
        }
    }

    #[inline]
    fn minus_unit(self, step: u8) -> Self {
        if self.is_eval() {
            Self::from_repr(self._repr - UNIT_SIZES[min(step as usize, UNIT_SIZES.len() - 1)])
        } else {
            self
        }
    }

    #[inline]
    fn null_window(self) -> (Self, Self) {
        if self.is_eval() {
            (Self::from_repr(self._repr), Self::from_repr(self._repr + 1))
        } else {
            (self, self.plus_one_step())
        }
    }
}

impl EvalTrait for Eval {
    const MIN: Eval = Eval::from_repr_const(-MAX_WINNER_FOUND_VALUE);
    const MAX: Eval = Eval::from_repr_const(MAX_WINNER_FOUND_VALUE);

    // TODO add/subtract WINNER_UNIT instead
    #[inline]
    fn plus_one_step(self) -> Eval {
        if self.is_eval() {
            self
        } else {
            let n = self.get_winner_steps();
            Self::from_winner_steps(n + n.signum())
        }
    }
}

impl std::fmt::Debug for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_eval() {
            let v = self._repr;
            write!(f, "Ev({:+3}.{:1})", v / 10, (v % 10).abs())
        } else {
            write!(f, "Win({:+3})", self.get_winner_steps())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_from_eval(e in any::<HV>()) {
            assert_eq!(e.max(-WINNER_UNIT + 1).min(WINNER_UNIT - 1), Eval::from_eval(e).get_eval())
        }

        #[test]
        fn test_from_winner_steps_and_get_winner_steps(n in (-MAX_WINNER_STEPS + 1)..=(MAX_WINNER_STEPS - 1)) {
            prop_assume!(n != 0);
            assert_eq!(n as i8, Eval::from_winner_steps(n as i8).get_winner_steps())
        }
    }

    #[test]
    fn test_from_eval_cases() {
        assert_eq!(0, Eval::from_eval(0).get_eval());
        assert_eq!(-1, Eval::from_eval(-1).get_eval());
        assert_eq!(5, Eval::from_eval(5).get_eval());
        assert_eq!(-5, Eval::from_eval(-5).get_eval());
    }

    #[test]
    fn test_from_winner_steps_cases() {
        assert_eq!(0, Eval::from_winner_steps(0).get_winner_steps());
        assert_eq!(-1, Eval::from_winner_steps(-1).get_winner_steps());
        assert_eq!(5, Eval::from_winner_steps(5).get_winner_steps());
        assert_eq!(-5, Eval::from_winner_steps(-5).get_winner_steps());
        assert_eq!(
            (MAX_WINNER_STEPS - 1) as i8,
            Eval::from_winner_steps(MAX_WINNER_STEPS as i8).get_winner_steps()
        );
        assert_eq!(
            (-MAX_WINNER_STEPS + 1) as i8,
            Eval::from_winner_steps((-MAX_WINNER_STEPS) as i8).get_winner_steps()
        );
        assert_eq!(1, Eval::win().get_winner_steps());
        assert_eq!(-1, Eval::lose().get_winner_steps());
    }

    #[test]
    fn test_plus_one_step() {
        assert_eq!(3, Eval::from_winner_steps(2).plus_one_step().get_winner_steps());
        assert_eq!(-4, Eval::from_winner_steps(-3).plus_one_step().get_winner_steps());
    }
}
