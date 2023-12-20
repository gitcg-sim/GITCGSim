use std::{fmt::Debug, ops::Neg};

use rand::rngs::ThreadRng;

use crate::{types::game_state::PlayerId, zobrist_hash::HashValue};

use super::PV;

pub trait ValueTrait: Sized + Send + Sync + Debug + Clone + Copy + PartialEq + Eq {}

#[cfg(feature = "serde")]
pub trait EvalTrait:
    ValueTrait + Default + PartialOrd + Ord + Neg<Output = Self> + serde::Serialize + for<'de> serde::Deserialize<'de>
{
    const MIN: Self;
    const MAX: Self;
    fn plus_one_step(self) -> Self {
        self
    }
}

#[cfg(not(feature = "serde"))]
pub trait EvalTrait: ValueTrait + Default + PartialOrd + Ord + Neg<Output = Self> {
    const MIN: Self;
    const MAX: Self;
    fn plus_one_step(self) -> Self {
        self
    }
}

pub trait Windowable: Sized {
    fn aspiration_window(self) -> (Self, Self);

    fn plus_unit(self, step: u8) -> Self;

    fn minus_unit(self, step: u8) -> Self;

    fn null_window(self) -> (Self, Self);
}

pub trait ZobristHashable {
    fn zobrist_hash(&self) -> HashValue;
}

pub trait Game: ZobristHashable + Debug + Clone + Send + Sync {
    #[cfg(feature = "serde")]
    type Action: ValueTrait + std::hash::Hash + serde::Serialize + for<'de> serde::Deserialize<'de>;
    #[cfg(not(feature = "serde"))]
    type Action: ValueTrait + std::hash::Hash;
    type Actions: IntoIterator<Item = Self::Action> + FromIterator<Self::Action>;
    type Error: Debug;
    type Eval: Windowable + EvalTrait;

    const PREPARE_FOR_EVAL: bool = false;

    fn winner(&self) -> Option<PlayerId>;

    fn to_move(&self) -> Option<PlayerId>;

    fn actions(&self) -> Self::Actions;

    fn advance(&mut self, action: Self::Action) -> Result<(), Self::Error>;

    /// Called before the game state is sent to the search algorithm.
    /// Modify the game state so the search algorithm cannot assume anything about the hidden information.
    fn hide_private_information(&mut self, player_to_hide: PlayerId);

    /// Called to prepare this game state for tactical search.
    fn convert_to_tactical_search(&mut self) {}

    /// Called to prepare this game state before static evaluation.
    fn prepare_for_eval(&mut self) {}

    fn eval(&self, player_id: PlayerId) -> Self::Eval;

    fn static_search_action(&self, player_id: PlayerId) -> Option<Self::Action>;

    fn action_weights(&self, actions: &Self::Actions) -> Vec<(Self::Action, f32)>;

    fn shuffle_actions(actions: &mut Self::Actions, rng: &mut ThreadRng);

    /// Apply move ordering.
    /// The move for the principal variation is moved to the front, then
    /// the moves similar to the PV (same kind: such as switching characters/playing card/casting skill)
    /// comes after.
    fn move_ordering(&self, pv: &PV<Self>, actions: &mut Self::Actions);

    fn round_number(&self) -> u8;

    fn is_tactical_action(action: Self::Action) -> bool;

    #[allow(unused_variables)]
    fn depth_extension(&self, action: Self::Action) -> u8 {
        0
    }
}
