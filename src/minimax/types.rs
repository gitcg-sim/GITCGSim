use std::cmp::min;

use serde::{Deserialize, Serialize};

use super::*;

/// Heuristic value
pub type HV = i16;

/// A game state evaluation, which is based on number of moves until win/lose, and heuristic score
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Eval {
    pub winner_found_value: i8,
    pub heuristic_value: HV,
}

const UNIT_SIZES: [HV; 4] = [4, 30, 60, 220];

impl Windowable for Eval {
    #[inline]
    fn aspiration_window(self) -> (Self, Self) {
        if self == Self::MIN || self == Self::MAX {
            (Self::MIN, Self::MAX)
        } else if self.winner_found_value == 0 {
            (
                Self {
                    winner_found_value: 0,
                    heuristic_value: self.heuristic_value - UNIT_SIZES[0],
                },
                Self {
                    winner_found_value: 0,
                    heuristic_value: self.heuristic_value + UNIT_SIZES[0],
                },
            )
        } else {
            (
                Self {
                    winner_found_value: self.winner_found_value - 1,
                    heuristic_value: self.heuristic_value,
                },
                Self {
                    winner_found_value: self.winner_found_value + 1,
                    heuristic_value: self.heuristic_value,
                },
            )
        }
    }

    #[inline]
    fn plus_unit(self, step: u8) -> Self {
        if self.winner_found_value == 0 {
            Self {
                winner_found_value: 0,
                heuristic_value: self.heuristic_value + UNIT_SIZES[min(step as usize, UNIT_SIZES.len() - 1)],
            }
        } else {
            Self {
                winner_found_value: self.winner_found_value + 1,
                heuristic_value: self.heuristic_value,
            }
        }
    }

    #[inline]
    fn minus_unit(self, step: u8) -> Self {
        if self.winner_found_value == 0 {
            Self {
                winner_found_value: 0,
                heuristic_value: self.heuristic_value - UNIT_SIZES[min(step as usize, UNIT_SIZES.len() - 1)],
            }
        } else {
            Self {
                winner_found_value: self.winner_found_value - 1,
                heuristic_value: self.heuristic_value,
            }
        }
    }

    #[inline]
    fn null_window(self) -> (Self, Self) {
        if self.winner_found_value == 0 {
            (
                Self {
                    winner_found_value: 0,
                    heuristic_value: self.heuristic_value,
                },
                Self {
                    winner_found_value: 0,
                    heuristic_value: self.heuristic_value + 1,
                },
            )
        } else {
            (
                Self {
                    winner_found_value: self.winner_found_value,
                    heuristic_value: self.heuristic_value,
                },
                Self {
                    winner_found_value: self.winner_found_value + 1,
                    heuristic_value: self.heuristic_value,
                },
            )
        }
    }
}

impl std::fmt::Debug for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Eval")
            .field(&self.winner_found_value)
            .field(&self.heuristic_value)
            .finish()
    }
}
