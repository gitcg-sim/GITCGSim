use super::dice_counter::DiceCounter;
use super::game_state::{CardSelection, CharSelection, PlayerId};
use crate::cards::ids::*;
use crate::data_structures::List8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlayerAction {
    EndRound,
    PlayCard(CardId, Option<CardSelection>),
    ElementalTuning(CardId),
    CastSkill(SkillId),
    SwitchCharacter(CharSelection),
    PostDeathSwitch(CharSelection),
}

/// A non-deterministic action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NondetResult {
    // TODO use ByPlayer
    ProvideDice(DiceCounter, DiceCounter),
    // TODO use ByPlayer
    ProvideCards(List8<CardId>, List8<CardId>),
    ProvideSummonIds(List8<SummonId>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Input {
    NoAction,
    NondetResult(NondetResult),
    FromPlayer(PlayerId, PlayerAction),
}

impl crate::std_subset::fmt::Debug for Input {
    fn fmt(&self, f: &mut crate::std_subset::fmt::Formatter<'_>) -> crate::std_subset::fmt::Result {
        match self {
            Self::FromPlayer(PlayerId::PlayerFirst, arg1) => f.debug_tuple("P0").field(arg1).finish(),
            Self::FromPlayer(PlayerId::PlayerSecond, arg1) => f.debug_tuple("P1").field(arg1).finish(),
            Self::NoAction => write!(f, "NoAction"),
            Self::NondetResult(arg0) => f.debug_tuple("NondetResult").field(arg0).finish(),
        }
    }
}

impl Input {
    #[inline]
    pub fn player(&self) -> Option<PlayerId> {
        match self {
            Input::NoAction => None,
            Input::NondetResult(..) => None,
            Input::FromPlayer(p, _) => Some(*p),
        }
    }

    #[inline]
    pub fn player_input(&self) -> Option<PlayerAction> {
        match self {
            Input::NoAction => None,
            Input::NondetResult(..) => None,
            Input::FromPlayer(_, i) => Some(*i),
        }
    }
}
