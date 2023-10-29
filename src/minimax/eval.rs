use crate::types::game_state::{CharState, PlayerState};

use super::HV;

#[inline]
fn low_hp_factor(hp: u8) -> HV {
    match hp {
        0 => 30,
        1 => 30 + 15,
        2 => 30 + 15 + 10,
        3 => 30 + 15 + 10 + 5,
        _ => 30 + 15 + 10 + 5 + 5,
    }
}

impl CharState {
    #[inline]
    fn active_eval(&self) -> HV {
        100 - 6 * (self.applied.len() as HV) + low_hp_factor(self.get_hp()) / 5
    }
}

impl PlayerState {
    #[allow(clippy::identity_op)]
    pub fn eval(&self) -> HV {
        let dice_value = self.dice.total();
        let hand_value = self.hand.len();
        let hp_total: u8 = self.char_states.iter_valid().map(CharState::get_hp).sum();
        let energy_total: u8 = self.char_states.iter_valid().map(CharState::get_energy).sum();
        let support_total = self.status_collection.support_count() as HV;
        let status_total = self.status_collection.status_count() as HV;
        let summons_total = self.status_collection.summon_count() as HV;
        let elem_total = self
            .char_states
            .iter_valid()
            .fold(0, |x, c| x + (c.applied.len() as u8));
        let low_hp_total = self
            .char_states
            .iter_valid()
            .fold(0, |x, c| x + low_hp_factor(c.get_hp()));
        // TODO use switch score instead
        let active_char_value = self.get_active_character().active_eval();

        0 + low_hp_total + 10 * (hp_total as HV) + 5 * (energy_total as HV) - 16 * (elem_total as HV)
            + active_char_value
            + 6 * (dice_value as HV)
            + 15 * support_total
            + 11 * status_total
            + 13 * summons_total
            + 2 * (hand_value as HV)
    }
}
