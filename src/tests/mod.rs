use crate::builder::*;
use crate::cards::ids::*;
use crate::dispatcher_ops::*;
use crate::types::{dice_counter::*, game_state::*, input::*, tcg_model::*};
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
        self.advance(Input::NondetResult(NondetResult::ProvideCards(Default::default())))
            .unwrap();
        self.advance(Input::NondetResult(NondetResult::ProvideDice(Default::default())))
            .unwrap();
    }

    /// Panics: If `advance` causes an error.
    fn advance_multiple<T: IntoIterator<Item = Input>>(self: &mut GameState, inputs: T) {
        for input in inputs.into_iter() {
            self.advance(input).unwrap();
        }
    }

    fn get_status_collection_mut(&mut self, player_id: PlayerId) -> &mut StatusCollection {
        self.status_collections.get_mut(player_id)
    }

    fn get_dice_distribution(&self, player_id: PlayerId) -> DiceDistribution {
        self.get_player(player_id)
            .get_dice_distribution(self.get_status_collection(player_id))
    }

    fn has_summon(&self, player_id: PlayerId, summon_id: SummonId) -> bool {
        self.status_collections.get(player_id).has_summon(summon_id)
    }

    fn has_team_status(&self, player_id: PlayerId, status_id: StatusId) -> bool {
        self.status_collections.get(player_id).has_team_status(status_id)
    }

    fn has_character_status(&self, player_id: PlayerId, char_idx: u8, status_id: StatusId) -> bool {
        self.status_collections
            .get(player_id)
            .has_character_status(char_idx, status_id)
    }

    fn has_active_character_status(&self, player_id: PlayerId, status_id: StatusId) -> bool {
        self.status_collections
            .get(player_id)
            .has_character_status(self.players.get(player_id).active_char_idx, status_id)
    }
}

impl PlayerState {
    /// Add card to hand, ignoring the result.
    pub fn add_to_hand_ignore(&mut self, card_id: CardId) {
        let _ = self.hand.push(card_id);
    }
}
