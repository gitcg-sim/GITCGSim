#![allow(non_snake_case)]
use std::fmt::{Debug, Display};

use enum_map::Enum;
use enumset::{enum_set, EnumSet, EnumSetType};

use smallvec::SmallVec;

use crate::cards::ids::lookup::GetStatus;
use crate::cards::ids::{CardId, SupportId};
use crate::data_structures::{CommandList, StatusEntryList, Vector};

use crate::cards::ids::*;
use crate::dispatcher_ops::types::NondetRequest;
use crate::vector;
use crate::zobrist_hash::ZobristHasher;

use super::by_player::ByPlayer;
use super::card_defs::Status;
use super::command::{EventId, XEventMask};
use super::enums::EquipSlot;
use super::status_impl::RespondsTo;
use super::{
    command::{Command, CommandContext},
    dice_counter::DiceCounter,
    logging::EventLog,
};

pub use super::applied_effect_state::AppliedEffectState;
pub use crate::types::card_selection::*;
pub use crate::types::char_state::*;

/// The deterministic and perfect information portion of the Genius Invokation TCG game state.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameState {
    /// When game state is suspended while executing commands
    pub pending_cmds: Option<Box<PendingCommands>>,
    pub round_number: u8,
    pub phase: Phase,
    /// 0 (PlayerFirst) goes first at turn 1
    pub players: ByPlayer<PlayerState>,
    pub log: Box<EventLog>,
    /// If this field is set to `true`, costs (dice and energy) will not
    /// be checked and will not be paid. Effects that reduce costs will never be consumed.
    pub ignore_costs: bool,
    /// The incrementally-updated portion of the Zobrist hash of this `GameState`.
    pub _incremental_hash: ZobristHasher,
    /// The entire Zobrist hash of this `GmaeState`.
    pub _hash: ZobristHasher,
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    pub active_char_idx: u8,
    pub dice: DiceCounter,
    pub char_states: CharStates,
    pub status_collection: StatusCollection,
    // TODO enforce limit of 10
    pub hand: Vector<CardId>,
    pub flags: EnumSet<PlayerFlag>,
    // TODO
    // pub taken_most_dmg: Option<TakenMostDMG>,
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

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A player's summons and applied statuses (team/characters)
pub struct StatusCollection {
    pub responds_to: EnumSet<RespondsTo>,
    pub responds_to_triggers: EnumSet<EventId>,
    pub responds_to_events: XEventMask,
    pub _status_entries: StatusEntryList<StatusEntry>,
}

#[derive(Debug, Default)]
pub enum CharIdxSelector {
    #[default]
    None,
    One(u8),
    All,
}

impl CharIdxSelector {
    #[inline]
    pub fn selects(&self, char_idx: u8) -> bool {
        match self {
            Self::None => false,
            Self::One(ci) => *ci == char_idx,
            Self::All => true,
        }
    }
}

impl From<Option<u8>> for CharIdxSelector {
    #[inline]
    fn from(value: Option<u8>) -> Self {
        match value {
            None => Self::None,
            Some(i) => Self::One(i),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SupportSlot {
    Slot0 = 0,
    Slot1 = 1,
    Slot2 = 2,
    Slot3 = 3,
}

impl SupportSlot {
    pub const VALUES: [Self; 4] = [Self::Slot0, Self::Slot1, Self::Slot2, Self::Slot3];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatusKey {
    Team(StatusId),
    Character(u8, StatusId),
    Equipment(u8, EquipSlot, StatusId),
    Summon(SummonId),
    Support(SupportSlot, SupportId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKeyFilter {
    Team,
    Character(u8),
    Equipment(u8, EquipSlot),
    Summon,
    Support(SupportSlot),
}

impl StatusKeyFilter {
    #[inline]
    pub fn matches(self, status_key: StatusKey) -> bool {
        match self {
            Self::Team => matches!(status_key, StatusKey::Team(..)),
            Self::Summon => matches!(status_key, StatusKey::Summon(..)),
            Self::Character(i) => {
                if let StatusKey::Character(j, ..) = status_key {
                    i == j
                } else {
                    false
                }
            }
            Self::Equipment(i, slot) => {
                if let StatusKey::Equipment(j, s, ..) = status_key {
                    i == j && s == slot
                } else {
                    false
                }
            }
            Self::Support(slot) => {
                if let StatusKey::Support(slot1, _) = status_key {
                    slot == slot1
                } else {
                    false
                }
            }
        }
    }
}

impl StatusKey {
    #[inline]
    pub fn status_id(&self) -> Option<StatusId> {
        match self {
            Self::Team(status_id) | Self::Character(_, status_id) | Self::Equipment(_, _, status_id) => {
                Some(*status_id)
            }
            Self::Summon(..) | Self::Support(..) => None,
        }
    }

    #[inline]
    pub fn is_equipment(&self) -> bool {
        matches!(self, Self::Equipment(..))
    }

    #[inline]
    pub fn summon_id(&self) -> Option<SummonId> {
        match self {
            Self::Team(..) | Self::Character(..) | Self::Equipment(..) | Self::Support(..) => None,
            Self::Summon(summon_id) => Some(*summon_id),
        }
    }

    #[inline]
    pub fn support_id(&self) -> Option<SupportId> {
        match self {
            Self::Support(_, support_id) => Some(*support_id),
            Self::Team(..) | Self::Character(..) | Self::Equipment(..) | Self::Summon(..) => None,
        }
    }

    #[inline]
    pub fn get_status(&self) -> &'static Status {
        match self {
            Self::Team(status_id) | Self::Character(_, status_id) | Self::Equipment(_, _, status_id) => {
                let status_id = *status_id;
                status_id.get_status()
            }
            Self::Summon(summon_id) => summon_id.get_status(),
            Self::Support(_, support_id) => {
                let support_id = *support_id;
                support_id.get_status()
            }
        }
    }

    #[inline]
    pub fn char_idx(&self) -> Option<u8> {
        match *self {
            Self::Character(char_idx, _) | Self::Equipment(char_idx, _, _) => Some(char_idx),
            _ => None,
        }
    }

    #[inline]
    pub fn sort_key(&self) -> u8 {
        match *self {
            Self::Equipment(..) => 0,
            Self::Character(..) => 1,
            Self::Team(..) => 2,
            Self::Summon(..) => 3,
            Self::Support(..) => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatusEntry {
    pub key: StatusKey,
    pub state: AppliedEffectState,
}

impl StatusEntry {
    #[inline]
    pub fn new(key: StatusKey, state: AppliedEffectState) -> Self {
        Self { key, state }
    }

    #[inline]
    pub fn support_id(self) -> Option<SupportId> {
        if let StatusKey::Support(_, support_id) = self.key {
            Some(support_id)
        } else {
            None
        }
    }

    #[inline]
    pub fn summon_id(self) -> Option<SummonId> {
        if let StatusKey::Summon(summon_id) = self.key {
            Some(summon_id)
        } else {
            None
        }
    }

    #[inline]
    pub fn status_id(self) -> Option<StatusId> {
        if let StatusKey::Character(_, status_id) | StatusKey::Team(status_id) = self.key {
            Some(status_id)
        } else {
            None
        }
    }
}
