use crate::std_subset::cmp::{max, min};

/// State variable for an applied effect (status or summon).
#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(from = "builder::AppliedEffectStateBuilder"),
    serde(into = "builder::AppliedEffectStateBuilder")
)]
pub struct AppliedEffectState {
    _repr: u8,
}

pub(crate) mod builder;

// Data layout
//  _repr           :  ? ? ? ? ? ? ? ?
//  usages          :            1 1 1
//  duration        :            1 1 1
//  counter         :    1 1 1 1
//  times per round :    1 1 1 1
//  once per round  :  1

const ONCE_PER_ROUND_MASK: u8 = 0b1000_0000_u8;
const USAGES_MASK: u8 = 0b0000_0111_u8;
const DURATION_MASK: u8 = 0b0000_0111_u8;
const COUNTER_MASK: u8 = 0b0111_1000_u8;
const COUNTER_SHIFT: u8 = 3;

impl From<u8> for AppliedEffectState {
    fn from(value: u8) -> Self {
        Self { _repr: value }
    }
}

impl AppliedEffectState {
    pub const DEFAULT: Self = Self { _repr: 0 };
    pub const MAX_COUNTER: u8 = 15;
    pub const MAX_USAGES: u8 = 7;
    pub const MAX_DURATION: u8 = 7;

    #[inline]
    pub(crate) const fn from_decomposed(once_per_round: bool, counter: u8, usages_or_duration: u8) -> Self {
        let mut val = 0;
        val |= usages_or_duration & USAGES_MASK;
        val |= (counter << COUNTER_SHIFT) & COUNTER_MASK;
        val |= (once_per_round as u8) * ONCE_PER_ROUND_MASK;
        Self { _repr: val }
    }

    #[inline]
    pub(crate) const fn decompose(self) -> (bool, u8, u8) {
        let value = self._repr;
        (
            value & ONCE_PER_ROUND_MASK != 0,
            (value & COUNTER_MASK) >> COUNTER_SHIFT,
            value & USAGES_MASK,
        )
    }

    /// Panics: when inputs are out of range
    #[inline]
    pub fn from_fields(usages: u8, duration: u8, once_per_round: bool) -> Self {
        let usages = min(Self::MAX_USAGES, usages);
        let duration = min(Self::MAX_DURATION, duration);
        if usages > 0 && duration > 0 {
            panic!("AppliedEffectState: usages and duration must be mutually exclusive")
        }
        let mut val = 0;
        val |= usages & USAGES_MASK;
        val |= duration & DURATION_MASK;
        if once_per_round {
            val |= ONCE_PER_ROUND_MASK
        }
        Self { _repr: val }
    }

    #[inline]
    pub fn get_usages(self) -> u8 {
        self._repr & USAGES_MASK
    }

    #[inline]
    pub fn get_counter(self) -> u8 {
        (self._repr & COUNTER_MASK) >> COUNTER_SHIFT
    }

    #[inline]
    pub fn set_counter(&mut self, counter: u8) {
        let counter = min(Self::MAX_COUNTER, counter);
        self._repr = (self._repr & !COUNTER_MASK) | ((counter << COUNTER_SHIFT) & COUNTER_MASK);
    }

    #[inline]
    pub fn get_duration(self) -> u8 {
        self._repr & DURATION_MASK
    }

    #[inline]
    pub fn set_usages(&mut self, usages: u8) {
        let usages = min(Self::MAX_USAGES, usages);
        self._repr = (self._repr & !USAGES_MASK) | usages
    }

    #[inline]
    pub fn set_duration(&mut self, duration: u8) {
        let duration = min(Self::MAX_DURATION, duration);
        self._repr = (self._repr & !DURATION_MASK) | duration
    }

    #[inline]
    pub fn no_duration(self) -> bool {
        self._repr & DURATION_MASK == 0
    }

    #[inline]
    pub fn no_usages(self) -> bool {
        self._repr & USAGES_MASK == 0
    }

    #[inline]
    pub fn can_use_once_per_round(self) -> bool {
        self._repr & ONCE_PER_ROUND_MASK != 0
    }

    #[inline]
    pub fn decrement_usages(&mut self) {
        let u = max(1, self.get_usages()) - 1;
        self.set_usages(u);
    }

    #[inline]
    pub fn decrement_duration(&mut self) {
        let d = max(1, self.get_duration()) - 1;
        self.set_duration(d);
    }

    #[inline]
    pub fn set_once_per_round_true(&mut self) {
        self._repr |= ONCE_PER_ROUND_MASK;
    }

    #[inline]
    pub fn set_once_per_round_false(&mut self) {
        self._repr &= !ONCE_PER_ROUND_MASK;
    }
}

impl crate::std_subset::fmt::Debug for AppliedEffectState {
    fn fmt(&self, f: &mut crate::std_subset::fmt::Formatter<'_>) -> crate::std_subset::fmt::Result {
        f.debug_struct("AppliedEffectState")
            .field("usages_duration", &self.get_usages())
            .field("counter", &self.get_counter())
            .field("once_per_round", &self.can_use_once_per_round())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_decrement_usages() {
        let mut s = AppliedEffectState::from_fields(3, 0, true);
        assert!(!s.no_usages());
        assert_eq!(3, s.get_usages());
        s.decrement_usages();
        assert_eq!(2, s.get_usages());
        s.decrement_usages();
        assert_eq!(1, s.get_usages());
        s.decrement_usages();
        assert_eq!(0, s.get_usages());
        assert!(s.no_usages());
    }

    #[test]
    fn test_decrement_usages_no_once_per_round() {
        let mut s = AppliedEffectState::from_fields(3, 0, false);
        assert_eq!(3, s.get_usages());
        s.decrement_usages();
        assert_eq!(2, s.get_usages());
        s.decrement_usages();
        assert_eq!(1, s.get_usages());
        s.decrement_usages();
        assert_eq!(0, s.get_usages());
    }

    #[test]
    fn test_set_usages() {
        let mut s = AppliedEffectState::from_fields(3, 0, true);
        assert_eq!(3, s.get_usages());
        s.set_usages(5);
        assert_eq!(5, s.get_usages());
    }

    #[test]
    fn test_set_counter() {
        let mut s = AppliedEffectState::from_fields(3, 0, true);
        assert!(s.can_use_once_per_round());
        assert_eq!(0, s.get_counter());
        assert_eq!(3, s.get_usages());
        s.set_counter(11);
        assert!(s.can_use_once_per_round());
        assert_eq!(11, s.get_counter());
        assert_eq!(3, s.get_usages());
    }
}
