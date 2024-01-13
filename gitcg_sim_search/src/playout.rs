use std::cell::RefCell;

use gitcg_sim::prelude::*;

use crate::{Game, GameTreeSearch};

#[derive(Debug, Copy, Clone)]
pub enum PlayoutError<E> {
    WinnerDecided(PlayerId),
    SearchError,
    BorrowSearchesError,
    AdvanceError(E),
    TerminatedByError,
}

/// A playout is a sequence of moves and game states (`G`) generated by the given search algorithms (`ByPlayer<S>`).
/// This type is a builder for the iterable type `PlayoutState`.
pub struct Playout<'a, G: Game, S: GameTreeSearch<G>> {
    pub max_steps: usize,
    pub initial_state: G,
    pub initial_searches: &'a RefCell<ByPlayer<S>>,
}

pub struct PlayoutState<'a, G: Game, S: GameTreeSearch<G>> {
    pub game_state: G,
    pub searches: &'a RefCell<ByPlayer<S>>,
    pub last_error: bool,
}

#[allow(type_alias_bounds)]
type PlayoutItem<'a, G: Game, S: GameTreeSearch<G>> =
    Result<(G::Action, G, &'a RefCell<ByPlayer<S>>), PlayoutError<G::Error>>;

impl<'a, G: Game, S: GameTreeSearch<G>> PlayoutState<'a, G, S> {
    fn advance(&mut self) -> PlayoutItem<'a, G, S> {
        if self.last_error {
            return Err(PlayoutError::TerminatedByError);
        }
        if let Some(winner) = self.game_state.winner() {
            return Err(PlayoutError::WinnerDecided(winner));
        }
        let player_id = self.game_state.to_move().expect("Player to move required here");
        let Ok(mut ref_searches) = self.searches.try_borrow_mut() else {
            return Err(PlayoutError::BorrowSearchesError);
        };
        let model = ref_searches.get_mut(player_id);
        let res = model.search_hidden(&self.game_state, player_id);
        let act = if let Some(h) = res.pv.head() {
            Ok(h)
        } else {
            Err(PlayoutError::SearchError)
        }?;
        self.game_state.advance(act).map_err(PlayoutError::AdvanceError)?;
        Ok((act, self.game_state.clone(), self.searches))
    }
}

impl<'a, G: Game, S: GameTreeSearch<G>> Iterator for PlayoutState<'a, G, S> {
    type Item = PlayoutItem<'a, G, S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_error {
            return None;
        }
        match self.advance() {
            Ok(res) => Some(Ok(res)),
            Err(PlayoutError::WinnerDecided(..)) => {
                self.last_error = true;
                None
            }
            Err(e) => {
                self.last_error = true;
                Some(Err(e))
            }
        }
    }
}

pub struct IterPlayout<'a, G: Game, S: GameTreeSearch<G>>(std::iter::Take<PlayoutState<'a, G, S>>);

impl<'a, G: Game, S: GameTreeSearch<G>> Iterator for IterPlayout<'a, G, S> {
    type Item = PlayoutItem<'a, G, S>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, G: Game, S: GameTreeSearch<G>> Playout<'a, G, S> {
    pub fn new(max_steps: usize, initial_state: G, initial_searches: &'a RefCell<ByPlayer<S>>) -> Self {
        Self {
            max_steps,
            initial_state,
            initial_searches,
        }
    }

    pub fn iter_playout(self) -> IterPlayout<'a, G, S> {
        IterPlayout(
            PlayoutState {
                game_state: self.initial_state,
                searches: self.initial_searches,
                last_error: false,
            }
            .take(self.max_steps),
        )
    }
}
