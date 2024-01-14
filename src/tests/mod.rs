use crate::builder::*;
use crate::cards::ids::*;
use crate::dispatcher_ops::types::*;
use crate::types::{dice_counter::*, enums::*, game_state::*, input::*};
use crate::{elem_set, list8, vector};
use enumset::enum_set;

pub mod characters;

pub mod duel_flow;

pub mod validation;

pub mod elemental_reactions;

pub mod cards;

pub mod zobrist_hash;

pub mod perf;

#[cfg(feature = "serde")]
pub mod serialization;

pub mod prop_tests;

pub const NO_ACTION: Input = Input::NoAction;

impl GameState {
    /// Panics: If `advance` causes an error.
    fn advance_roll_phase_no_dice(self: &mut GameState) {
        self.advance(NO_ACTION).unwrap();
        self.advance(Input::NondetResult(NondetResult::ProvideCards(list8![], list8![])))
            .unwrap();
        self.advance(Input::NondetResult(NondetResult::ProvideDice(
            DiceCounter::EMPTY,
            DiceCounter::EMPTY,
        )))
        .unwrap();
    }

    /// Panics: If `advance` causes an error.
    fn advance_multiple<T: IntoIterator<Item = Input>>(self: &mut GameState, inputs: T) {
        for input in inputs.into_iter() {
            self.advance(input).unwrap();
        }
    }
}

impl PlayerState {
    #[allow(dead_code)]
    fn has_summon(&self, summon_id: SummonId) -> bool {
        self.status_collection.has_summon(summon_id)
    }

    fn has_team_status(&self, status_id: StatusId) -> bool {
        self.status_collection.has_team_status(status_id)
    }

    pub fn has_character_status(&self, char_idx: u8, status_id: StatusId) -> bool {
        self.status_collection.has_character_status(char_idx, status_id)
    }

    pub fn has_active_character_status(&self, status_id: StatusId) -> bool {
        self.status_collection
            .has_character_status(self.active_char_idx, status_id)
    }
}
