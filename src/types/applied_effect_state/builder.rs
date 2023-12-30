use super::*;

crate::with_updaters!(
    #[derive(Default, Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct AppliedEffectStateBuilder {
        pub once_per_round: bool,
        pub counter: u8,
        pub usages_or_duration: u8,
    }
);

impl AppliedEffectStateBuilder {
    #[inline]
    pub const fn build(self) -> AppliedEffectState {
        AppliedEffectState::from_decomposed(self.once_per_round, self.counter, self.usages_or_duration)
    }
}

impl AppliedEffectState {
    #[inline]
    pub const fn into_builder(self) -> AppliedEffectStateBuilder {
        let (once_per_round, counter, usages_or_duration) = self.decompose();
        AppliedEffectStateBuilder {
            once_per_round,
            counter,
            usages_or_duration,
        }
    }
}

crate::impl_from_to_builder!(AppliedEffectState, AppliedEffectStateBuilder);
