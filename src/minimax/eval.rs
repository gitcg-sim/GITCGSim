use std::{cmp::Ordering, ops::Neg};

use crate::types::game_state::{CharState, PlayerState};

use super::{Eval, EvalTrait, HV};

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
        let hp_total = self.char_states.iter().fold(0, |x, c| x + c.get_hp());
        let energy_total = self
            .char_states
            .iter()
            .fold(0, |x, c| x + if c.get_hp() > 0 { c.get_energy() } else { 0 });
        let support_total = self.status_collection.support_count() as HV;
        let status_total = self.status_collection.status_count() as HV;
        let summons_total = self.status_collection.summon_count() as HV;
        let elem_total = self.char_states.iter().fold(0, |x, c| x + (c.applied.len() as u8));
        let low_hp_total = self.char_states.iter().fold(0, |x, c| x + low_hp_factor(c.get_hp()));
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

impl Neg for Eval {
    type Output = Self;

    #[inline]
    fn neg(self) -> Eval {
        Eval::new(-self.winner_found_value, -self.heuristic_value)
    }
}

impl EvalTrait for Eval {
    const MIN: Eval = Eval::new(-100, 0);
    const MAX: Eval = Eval::new(100, 0);

    #[inline]
    fn plus_one_turn(self) -> Eval {
        let Eval {
            winner_found_value: x,
            heuristic_value: y,
        } = self;
        match x.cmp(&0) {
            Ordering::Less => Eval::new(x + 1, y),
            Ordering::Equal => Eval::new(x, y),
            Ordering::Greater => Eval::new(x - 1, y),
        }
    }
}

impl Eval {
    #[inline]
    pub const fn new(winner_found_value: i8, heuristic_value: HV) -> Eval {
        Eval {
            winner_found_value,
            heuristic_value,
        }
    }

    #[inline]
    pub fn from_heuristic(e: HV) -> Eval {
        Eval::new(0, e)
    }

    #[inline]
    pub fn win(e: HV) -> Eval {
        Eval::new(Eval::MAX.winner_found_value, e)
    }

    #[inline]
    pub fn lose(e: HV) -> Eval {
        Eval::new(Eval::MIN.winner_found_value, e)
    }
}
