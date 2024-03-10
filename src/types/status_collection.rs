#![allow(non_snake_case)]
use crate::std_subset::fmt::Debug;

use enum_map::Enum;
use enumset::EnumSet;

use crate::data_structures::StatusEntryList;

use super::card_defs::Status;
use super::command::{EventId, XEventMask};
use super::status_impl::RespondsTo;
use super::tcg_model::EquipSlot;
use crate::cards::ids::*;

pub use super::applied_effect_state::AppliedEffectState;

#[derive(Debug, Clone, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(from = "StatusEntryList<StatusEntry>"),
    serde(into = "StatusEntryList<StatusEntry>")
)]
/// A player's summons and applied statuses (team/characters)
pub struct StatusCollection {
    pub(crate) responds_to: EnumSet<RespondsTo>,
    pub(crate) responds_to_triggers: EnumSet<EventId>,
    pub(crate) responds_to_events: XEventMask,
    pub(crate) status_entries: StatusEntryList<StatusEntry>,
}

impl StatusCollection {
    pub fn new<T: IntoIterator<Item = StatusEntry>>(value: T) -> Self {
        let mut sc = Self {
            responds_to: Default::default(),
            responds_to_triggers: Default::default(),
            responds_to_events: Default::default(),
            status_entries: value.into_iter().collect(),
        };
        sc.refresh_responds_to();
        sc
    }
}

impl<T: IntoIterator<Item = StatusEntry>> From<T> for StatusCollection {
    #[inline]
    fn from(value: T) -> Self {
        StatusCollection::new(value)
    }
}

impl From<StatusCollection> for StatusEntryList<StatusEntry> {
    #[inline]
    fn from(value: StatusCollection) -> Self {
        value.status_entries.clone()
    }
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
    pub fn status(&self) -> &'static Status {
        match self {
            Self::Team(status_id) | Self::Character(_, status_id) | Self::Equipment(_, _, status_id) => {
                let status_id = *status_id;
                status_id.status()
            }
            Self::Summon(summon_id) => summon_id.status(),
            Self::Support(_, support_id) => {
                let support_id = *support_id;
                support_id.status()
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
