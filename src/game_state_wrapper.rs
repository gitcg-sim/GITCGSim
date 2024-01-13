use rand::rngs::SmallRng;
use std::fmt::Debug;

use crate::{data_structures::ActionList, prelude::*};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct GameStateWrapper<S: NondetState = StandardNondetHandlerState> {
    pub game_state: GameState,
    #[cfg_attr(feature = "serde", serde(rename = "nondet"))]
    pub nd: NondetProvider<S>,
}

impl<S: NondetState> GameStateWrapper<S> {
    pub fn winner(&self) -> Option<PlayerId> {
        match self.game_state.phase {
            Phase::WinnerDecided { winner } => Some(winner),
            _ => None,
        }
    }

    pub fn to_move(&self) -> Option<PlayerId> {
        self.game_state.to_move_player()
    }

    pub fn advance(&mut self, action: Input) -> Result<(), DispatchError> {
        let _ = self.game_state.advance(action)?;
        self.ensure_player();
        Ok(())
    }

    pub fn actions(&self) -> ActionList<Input> {
        self.game_state.available_actions()
    }

    pub fn hide_private_information(&mut self, player_to_hide: PlayerId) {
        self.game_state.log.enabled = false;
        self.nd.hide_private_information(&mut self.game_state, player_to_hide);
    }
}

impl<S: NondetState> Debug for GameStateWrapper<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameStateWrapper")
            .field("game_state", &self.game_state)
            .finish()
    }
}

impl<S: NondetState> GameStateWrapper<S> {
    pub fn new(game_state: GameState, nd: NondetProvider<S>) -> Self {
        let mut new = Self { game_state, nd };
        new.ensure_player();
        new
    }

    pub fn ensure_player(&mut self) {
        while self.to_move().is_none() && self.winner().is_none() {
            let input = self.nd.get_no_to_move_player_input(&self.game_state);
            if let Err(e) = self.game_state.advance(input) {
                dbg!(&self.game_state);
                panic!("{e:?}\n{input:?}");
            }
        }
    }
}

pub fn new_standard_game(
    decklist1: &Decklist,
    decklist2: &Decklist,
    rng: SmallRng,
) -> GameStateWrapper<StandardNondetHandlerState> {
    let game_state = {
        GameStateBuilder::new(decklist1.characters.clone(), decklist2.characters.clone())
            .start_at_select_character()
            .enable_log(false)
            .build()
    };
    let state = StandardNondetHandlerState::new(decklist1, decklist2, rng.into());
    GameStateWrapper::new(game_state, NondetProvider::new(state))
}

impl<S: NondetState> ZobristHashable for GameStateWrapper<S> {
    #[inline]
    fn zobrist_hash(&self) -> u64 {
        self.game_state.zobrist_hash()
    }
}
