use super::{game_state::StatusKey, *};
use applied_effect_state::AppliedEffectState;
use smallvec::SmallVec;

#[derive(Default)]
pub struct StatusSpecModifier {
    pub modifiers: SmallVec<[(StatusKey, u8); 4]>,
}

impl StatusSpecModifier {
    #[inline]
    pub fn push(&mut self, key: StatusKey, count: u8) {
        self.modifiers.push((key, count));
    }

    #[inline]
    pub fn modify(&self, key: StatusKey, eff_state: &mut AppliedEffectState) {
        for (_, count) in self.modifiers.iter().copied().filter(|k| k.0 == key) {
            eff_state.set_usages(eff_state.usages() + count);
        }
    }
}
