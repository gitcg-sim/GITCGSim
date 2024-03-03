#![allow(non_snake_case)]
use enumset::{EnumSet, EnumSetType};

// TODO remove
#[derive(Debug, PartialOrd, Ord, EnumSetType)]
#[enumset(repr = "u8")]
pub enum CharIdx {
    I0 = 0,
    I1 = 1,
    I2 = 2,
    I3 = 3,
}

impl CharIdx {
    #[inline]
    pub fn value(self) -> u8 {
        self as isize as u8
    }
}

pub type CharIdxSet = EnumSet<CharIdx>;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RelativeCharIdx {
    Previous,
    Next,
    ClosestTo(u8),
    ImmediateNext,
}

impl TryFrom<u8> for CharIdx {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::I0),
            1 => Ok(Self::I1),
            2 => Ok(Self::I2),
            3 => Ok(Self::I3),
            _ => Err(()),
        }
    }
}

impl From<CharIdx> for u8 {
    #[inline]
    fn from(val: CharIdx) -> Self {
        val.value()
    }
}

impl RelativeCharIdx {
    pub fn indexing_seq(self, char_idx: u8, n: u8) -> impl Iterator<Item = u8> {
        let i0 = char_idx;
        (0..n).map(move |d| match self {
            RelativeCharIdx::Previous => {
                if d > n {
                    n
                } else {
                    (i0 + n - d - 1) % n
                }
            }
            RelativeCharIdx::Next => {
                if d > n {
                    n
                } else {
                    (i0 + d + 1) % n
                }
            }
            RelativeCharIdx::ClosestTo(mid) => {
                if mid >= n {
                    return n - 1 - d;
                }
                let dr = n - 1 - mid;
                let rev = d % 2 == 1;
                let k = 2 * dr.min(mid);
                let low = d <= k;
                if mid == 0 {
                    d
                } else if mid == n - 1 {
                    n - 1 - d
                } else if low {
                    let d = (d + 1) / 2;
                    if rev {
                        mid - d
                    } else {
                        mid + d
                    }
                } else if mid >= dr {
                    n - 1 - d
                } else {
                    d
                }
            }
            RelativeCharIdx::ImmediateNext => {
                if d == 0 {
                    (i0 + 1).min(n - 1)
                } else {
                    i0
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    const N_CHARS: u8 = 15;

    fn arb_relative_switch_type_expect_immediate_next() -> impl Strategy<Value = RelativeCharIdx> {
        prop_oneof![
            Just(RelativeCharIdx::Previous),
            Just(RelativeCharIdx::Next),
            (0..N_CHARS).prop_map(RelativeCharIdx::ClosestTo),
        ]
    }
    proptest! {
        #[test]
        fn indexing_seq_has_length_of_n(n in 1..N_CHARS, s in arb_relative_switch_type_expect_immediate_next(), char_idx in 0..N_CHARS) {
            prop_assume!(char_idx < n);
            assert_eq!(n, s.indexing_seq(char_idx, n).collect::<Vec<_>>().len() as u8);
        }

        #[test]
        fn indexing_seq_is_a_permutation_of_range_from_zero_to_n(n in 1..N_CHARS, s in arb_relative_switch_type_expect_immediate_next(), char_idx in 0..N_CHARS) {
            prop_assume!(char_idx < n);
            let perm = s.indexing_seq(char_idx, N_CHARS).collect::<Vec<_>>();
            let sorted = { let mut sorted = perm.clone(); sorted.sort(); sorted };
            assert_eq!((0..N_CHARS).collect::<Vec<_>>(), sorted, "{perm:?}, s={s:?} char_idx={char_idx}");
        }

        #[test]
        fn indexing_seq_for_closest_to(char_idx in 0..N_CHARS) {
            let perm = RelativeCharIdx::ImmediateNext.indexing_seq(char_idx, N_CHARS).take(2).collect::<Vec<_>>();
            if char_idx + 1 < N_CHARS {
                assert_eq!(vec![char_idx + 1, char_idx], perm);
            } else {
                assert_eq!(vec![char_idx, char_idx], perm);
            }
        }
    }
}
