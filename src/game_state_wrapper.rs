use crate::{std_subset::fmt::Debug, types::game_state::GameStateParams};
use rand::rngs::SmallRng;

use crate::{data_structures::ActionList, prelude::*};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct GameStateWrapper<S: NondetState = StandardNondetHandlerState, P: GameStateParams = ()> {
    pub game_state: GameState<P>,
    #[cfg_attr(feature = "serde", serde(rename = "nondet"))]
    pub nd: NondetProvider<S>,
}

impl<S: NondetState, P: GameStateParams> GameStateWrapper<S, P> {
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
        self.nd.hide_private_information(&mut self.game_state, player_to_hide);
    }

    pub fn with_log<L: EventLog, Q: GameStateParams<EventLog = L>>(self, log: L) -> GameStateWrapper<S, Q> {
        GameStateWrapper::<S, Q> {
            game_state: self.game_state.with_log(log),
            nd: self.nd,
        }
    }
}

impl<S: NondetState, P: GameStateParams> Debug for GameStateWrapper<S, P> {
    fn fmt(&self, f: &mut crate::std_subset::fmt::Formatter<'_>) -> crate::std_subset::fmt::Result {
        f.debug_struct("GameStateWrapper")
            .field("game_state", &self.game_state)
            .finish()
    }
}

impl<S: NondetState, P: GameStateParams> GameStateWrapper<S, P> {
    pub fn new(game_state: GameState<P>, nd: NondetProvider<S>) -> Self {
        let mut new = Self { game_state, nd };
        new.ensure_player();
        new
    }

    /// Panics: [GameState] advance error
    pub fn ensure_player(&mut self) {
        while self.to_move().is_none() && self.winner().is_none() {
            let input = self.nd.no_to_move_player_input(&self.game_state);
            if let Err(e) = self.game_state.advance(input) {
                panic!("{e:?}\n{input:?}");
            }
        }
    }
}

// TODO take ByPlayer instead
pub fn new_standard_game(
    decklists: ByPlayer<&Decklist>,
    rng: SmallRng,
) -> GameStateWrapper<StandardNondetHandlerState> {
    let ByPlayer(decklist1, decklist2) = decklists;
    let game_state = {
        GameStateInitializer::new(decklist1.characters.clone(), decklist2.characters.clone())
            .start_at_beginning()
            .enable_log(false)
            .build()
    };
    let state = StandardNondetHandlerState::new(decklist1, decklist2, rng.into());
    GameStateWrapper::new(game_state, NondetProvider::new(state))
}

impl<S: NondetState> ZobristHashable for GameStateWrapper<S> {
    #[inline]
    fn zobrist_hash(&self) -> HashValue {
        self.game_state.zobrist_hash()
    }
}
