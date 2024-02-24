#![allow(non_snake_case)]
use crate::std_subset::{
    fmt::{Debug, Display},
    Box,
};

use enum_map::Enum;
use enumset::{enum_set, EnumSet, EnumSetType};

use smallvec::SmallVec;

use crate::{
    cards::ids::*,
    data_structures::{CommandList, Vector},
    dispatcher_ops::NondetRequest,
    vector,
    zobrist_hash::ZobristHasher,
};

use super::by_player::ByPlayer;

use super::{
    command::{Command, CommandContext},
    dice_counter::DiceCounter,
    logging::EventLog,
};

pub use super::applied_effect_state::AppliedEffectState;
pub use super::card_selection::*;
pub use super::char_state::*;
pub use super::status_collection::*;

/// The deterministic and perfect information portion of the Genius Invokation TCG game state.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(from = "crate::builder::GameStateBuilder"),
    serde(into = "crate::builder::GameStateBuilder")
)]
pub struct GameState {
    /// When game state is suspended while executing commands
    pub(crate) pending_cmds: Option<Box<PendingCommands>>,
    pub(crate) round_number: u8,
    pub(crate) phase: Phase,
    /// 0 (PlayerFirst) goes first at turn 1
    pub(crate) players: ByPlayer<PlayerState>,

    // Transient states below

    // TODO use a Box<dyn> event log handler instead
    pub log: Box<EventLog>,
    /// If this field is set to `true`, costs (dice and energy) will not
    /// be checked and will not be paid. Effects that reduce costs will never be consumed.
    pub ignore_costs: bool,
    /// The incrementally-updated portion of the Zobrist hash of this `GameState`.
    pub(crate) _incremental_hash: ZobristHasher,
    /// The entire Zobrist hash of this `GmaeState`.
    pub(crate) _hash: ZobristHasher,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PendingCommands {
    pub suspended_state: SuspendedState,
    pub pending_cmds: CommandList<(CommandContext, Command)>,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SuspendedState {
    PostDeathSwitch {
        player_id: PlayerId,
        character_statuses_to_shift: [Option<StatusEntry>; 2],
    },
    NondetRequest(NondetRequest),
}

impl SuspendedState {
    #[inline]
    pub fn post_death_switch(player_id: PlayerId) -> Self {
        Self::PostDeathSwitch {
            player_id,
            character_statuses_to_shift: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Enum)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlayerId {
    #[default]
    PlayerFirst = 0,
    PlayerSecond = 1,
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut crate::std_subset::fmt::Formatter<'_>) -> crate::std_subset::fmt::Result {
        match self {
            PlayerId::PlayerFirst => f.write_fmt(format_args!("P1")),
            PlayerId::PlayerSecond => f.write_fmt(format_args!("P2")),
        }
    }
}

impl PlayerId {
    #[inline]
    pub fn opposite(self) -> PlayerId {
        match self {
            PlayerId::PlayerFirst => PlayerId::PlayerSecond,
            PlayerId::PlayerSecond => PlayerId::PlayerFirst,
        }
    }

    #[inline]
    pub fn select<T>(self, tuple: (T, T)) -> T {
        match self {
            PlayerId::PlayerFirst => tuple.0,
            PlayerId::PlayerSecond => tuple.1,
        }
    }

    #[inline]
    pub fn select_mut<T>(self, tuple: &mut (T, T)) -> &mut T {
        match self {
            PlayerId::PlayerFirst => &mut tuple.0,
            PlayerId::PlayerSecond => &mut tuple.1,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RollPhaseState {
    #[default]
    Start,
    Drawing,
    Rolling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelectStartingCharacterState {
    Start { to_select: PlayerId },
    FirstSelected { to_select: PlayerId },
}

impl Default for SelectStartingCharacterState {
    fn default() -> Self {
        Self::Start {
            to_select: PlayerId::PlayerFirst,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Phase {
    SelectStartingCharacter {
        state: SelectStartingCharacterState,
    },
    RollPhase {
        first_active_player: PlayerId,
        roll_phase_state: RollPhaseState,
    },
    ActionPhase {
        first_end_round: Option<PlayerId>,
        active_player: PlayerId,
    },
    EndPhase {
        next_first_active_player: PlayerId,
    },
    WinnerDecided {
        winner: PlayerId,
    },
}

impl SelectStartingCharacterState {
    #[inline]
    pub fn active_player(self) -> PlayerId {
        match self {
            Self::Start { to_select } => to_select,
            Self::FirstSelected { to_select } => to_select,
        }
    }
}

impl Phase {
    #[inline]
    pub fn new_roll_phase(first_active_player: PlayerId) -> Phase {
        Phase::RollPhase {
            first_active_player,
            roll_phase_state: RollPhaseState::Start,
        }
    }

    #[inline]
    pub fn active_player(&self) -> Option<PlayerId> {
        match self {
            Phase::SelectStartingCharacter { state } => Some(state.active_player()),
            Phase::ActionPhase { active_player, .. } => Some(*active_player),
            _ => None,
        }
    }

    #[inline]
    pub fn opponent_ended_round(&self, player_id: PlayerId) -> bool {
        match *self {
            Phase::ActionPhase {
                active_player,
                first_end_round,
            } => active_player == player_id && first_end_round == Some(player_id.opposite()),
            _ => false,
        }
    }

    #[inline]
    pub fn winner(&self) -> Option<PlayerId> {
        match self {
            Phase::WinnerDecided { winner } => Some(*winner),
            _ => None,
        }
    }
}

#[derive(Debug, PartialOrd, Ord, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[enumset(repr = "u8")]
pub enum PlayerFlag {
    ChargedAttack,
    DiedThisRound,
    SkillCastedThisMatch,
    Tactical,
}

impl PlayerFlag {
    pub const END_OF_TURN_CLEAR: EnumSet<Self> = enum_set![Self::DiedThisRound];
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerState {
    pub(crate) active_char_idx: u8,
    pub(crate) dice: DiceCounter,
    pub(crate) char_states: CharStates,
    pub(crate) status_collection: StatusCollection,
    // TODO enforce limit of 10
    pub(crate) hand: Vector<CardId>,
    pub(crate) flags: EnumSet<PlayerFlag>,
}

impl PlayerState {
    pub fn new<T: IntoIterator<Item = CharId>>(char_ids: T) -> Self {
        Self {
            dice: DiceCounter::default(),
            char_states: CharStates::from_ids(char_ids),
            active_char_idx: 0,
            status_collection: StatusCollection::default(),
            hand: vector![],
            flags: enum_set![],
        }
    }

    #[inline]
    pub fn is_tactical(&self) -> bool {
        self.flags.contains(PlayerFlag::Tactical)
    }

    #[inline]
    pub fn get_active_char_idx(&self) -> u8 {
        self.active_char_idx
    }

    #[inline]
    pub fn get_dice_counter(&self) -> DiceCounter {
        self.dice
    }

    #[inline]
    pub fn get_status_collection(&self) -> &StatusCollection {
        &self.status_collection
    }

    #[inline]
    pub fn hand_is_empty(&self) -> bool {
        self.hand.is_empty()
    }

    #[inline]
    pub fn hand_len(&self) -> u8 {
        self.hand.len() as u8
    }

    #[inline]
    pub fn get_hand(&self) -> &Vector<CardId> {
        &self.hand
    }

    #[inline]
    pub fn get_char_states(&self) -> &CharStates {
        &self.char_states
    }

    #[inline]
    pub fn get_flags(&self) -> EnumSet<PlayerFlag> {
        self.flags
    }
}

/// This type exists because when statuses are being modified, the entire
/// `PlayerState` except the `status_collection` need to be borrowed
/// immutably.
#[derive(Debug, Clone)]
pub struct PlayerStateView<'a> {
    pub active_char_idx: u8,
    pub char_states: &'a CharStates,
    pub flags: EnumSet<PlayerFlag>,
    pub dice: DiceCounter,
    pub affected_by: SmallVec<[StatusKey; 4]>,
}

impl<'a> PlayerStateView<'a> {
    pub fn active_char_state(&self) -> &CharState {
        &self.char_states[self.active_char_idx]
    }
}

impl GameState {
    #[inline]
    pub fn get_phase(&self) -> Phase {
        self.phase
    }

    #[inline]
    pub fn get_round_number(&self) -> u8 {
        self.round_number
    }
}
