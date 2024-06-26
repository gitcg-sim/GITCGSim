use crate::builder::*;
use crate::cards::ids::*;
use crate::dispatcher_ops::*;
use crate::types::{by_player::ByPlayer, dice_counter::*, game_state::*, input::*, tcg_model::*};

use crate::{action_list, elem_set, list8, vector};
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

impl<P: GameStateParams> crate::types::game_state::GameState<P> {
    /// Handle advancing the game state through RollPhase until the start of Action Phase
    /// Panics: If `advance` causes an error.
    fn advance_roll_phase_no_dice(&mut self) {
        assert!(
            !matches!(
                self.phase,
                Phase::ActionPhase { .. } | Phase::EndPhase { .. } | Phase::WinnerDecided { .. }
            ),
            "invalid phase: {:?}",
            self.phase
        );
        if matches!(self.phase, Phase::Drawing { .. }) {
            assert!(matches!(self.nondet_request(), Some(NondetRequest::DrawCards(..))));
            self.advance(Input::NondetResult(NondetResult::ProvideCards(Default::default())))
                .unwrap();
            if self.round_number == 1 {
                // TODO mulligan here
                assert!(
                    matches!(self.phase, Phase::SelectStartingCharacter { .. }),
                    "must be on Phase::SelectStartingCharacter"
                );
                assert!(self.nondet_request().is_none(), "must have no NondetRequest");
                self.advance(Input::FromPlayer(
                    PlayerId::PlayerFirst,
                    PlayerAction::SwitchCharacter(0),
                ))
                .unwrap();
                self.advance(Input::FromPlayer(
                    PlayerId::PlayerSecond,
                    PlayerAction::SwitchCharacter(0),
                ))
                .unwrap();
            }
        }

        // Roll Phase
        assert!(
            matches!(self.phase, Phase::RollPhase { .. }),
            "must be on Phase::RollPhase"
        );
        match self.phase {
            Phase::RollPhase {
                roll_phase_state: RollPhaseState::Start,
                ..
            } => {
                self.advance(Input::NoAction).unwrap();
            }
            Phase::RollPhase {
                roll_phase_state: RollPhaseState::Rolling,
                ..
            } => {
                // do nothing
            }
            _ => unreachable!(),
        }
        self.advance(Input::NondetResult(NondetResult::ProvideDice(Default::default())))
            .unwrap();
        assert!(
            matches!(self.phase, Phase::ActionPhase { .. }),
            "must be on Phase::ActionPhase"
        );
    }

    /// Panics: If `advance` causes an error.
    fn advance_multiple<T: IntoIterator<Item = Input>>(self: &mut crate::types::game_state::GameState<P>, inputs: T) {
        for input in inputs.into_iter() {
            self.advance(input).unwrap();
        }
    }

    fn status_collection_mut(&mut self, player_id: PlayerId) -> &mut StatusCollection {
        self.status_collections.get_mut(player_id)
    }

    fn dice_distribution(&self, player_id: PlayerId) -> DiceDistribution {
        self.player(player_id)
            .dice_distribution(self.status_collection(player_id))
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
