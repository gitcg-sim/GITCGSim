use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct Proportion {
    pub q: u32,
    pub n: u32,
}

impl Display for Proportion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}", self.q, self.n))
    }
}

impl Proportion {
    #[inline]
    pub fn new(q: u32, n: u32) -> Self {
        Self { q, n }
    }

    #[inline]
    pub fn ratio(self) -> f32 {
        ((self.q + 1) as f32) / ((self.n + 2) as f32)
    }

    pub fn sd(self) -> f32 {
        let r = self.ratio();
        f32::sqrt(r * (1.0 - r) / ((self.n + 2) as f32))
    }

    #[inline]
    pub fn complement(self) -> Self {
        Self::new(self.n - self.q, self.n)
    }
}

impl Add for Proportion {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.q + rhs.q, self.n + rhs.n)
    }
}

impl AddAssign for Proportion {
    fn add_assign(&mut self, rhs: Self) {
        self.q += rhs.q;
        self.n += rhs.n;
    }
}

impl From<(u32, u32)> for Proportion {
    #[inline]
    fn from(value: (u32, u32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Proportion> for (u32, u32) {
    fn from(p: Proportion) -> Self {
        (p.q, p.n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn arb_prop()(a in any::<u32>(), b in any::<u32>()) -> Proportion {
            Proportion::new(a, a.saturating_add(b))
        }
    }

    proptest! {
        #[test]
        fn test_complement_of_complement(p in arb_prop()) {
            assert_eq!(p, p.complement().complement())
        }
    }
}
