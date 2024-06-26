#![allow(non_snake_case)]
use crate::data_structures::capped_list::CapList;
use crate::std_subset::fmt::Debug;
use crate::std_subset::ops::{Index, IndexMut};

use constdefault::ConstDefault;
use enumset::{enum_set, EnumSet, EnumSetType};

use crate::cards::ids::*;

pub use crate::types::applied_effect_state::AppliedEffectState;
use crate::types::ElementSet;

use super::card_defs::CharCard;
use super::dice_counter::ElementPriority;

pub mod builder;

#[derive(Debug, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[enumset(repr = "u8")]
pub enum CharFlag {
    TalentEquipped,
    PlungingAttack,
    SkillCastedThisTurn0,
    SkillCastedThisTurn1,
    SkillCastedThisTurn2,
    SkillCastedThisTurn3,
}

impl CharFlag {
    pub const RETAIN: EnumSet<Self> = enum_set![Self::TalentEquipped];
}

#[derive(Copy, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(from = "builder::CharStateBuilder"),
    serde(into = "builder::CharStateBuilder")
)]
pub struct CharState {
    pub(crate) char_id: CharId,
    _hp_and_energy: u8,
    pub(crate) applied: ElementSet,
    pub(crate) flags: EnumSet<CharFlag>,
    pub(crate) total_dmg_taken: u8,

    #[cfg_attr(feature = "serde", serde(skip))]
    element_priority: Option<ElementPriority>,
}

impl Debug for CharState {
    fn fmt(&self, f: &mut crate::std_subset::fmt::Formatter<'_>) -> crate::std_subset::fmt::Result {
        f.debug_struct("CharState")
            .field("char_id", &self.char_id)
            .field("hp", &self.hp())
            .field("energy", &self.energy())
            .field("applied", &self.applied)
            .field("flags", &self.flags)
            .field("total_dmg_taken", &self.total_dmg_taken)
            .field("element_priority", &self.element_priority)
            .finish()
    }
}

impl ConstDefault for CharState {
    const DEFAULT: Self = Self {
        char_id: ConstDefault::DEFAULT,
        _hp_and_energy: ConstDefault::DEFAULT,
        applied: enum_set![],
        flags: enum_set![],
        total_dmg_taken: ConstDefault::DEFAULT,
        element_priority: ConstDefault::DEFAULT,
    };
}

#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
pub struct CharStates {
    char_states: CapList<CharState, { Self::MAX_CHAR_STATES }>,
}

impl CharStates {
    pub const MAX_CHAR_STATES: usize = 4;

    pub fn from_ids<T: IntoIterator<Item = CharId>>(char_ids: T) -> Self {
        let v: heapless::Vec<CharState, { Self::MAX_CHAR_STATES }> = char_ids.into_iter().map(CharState::new).collect();
        Self {
            char_states: CapList::from_slice_copy(&v),
        }
    }

    #[inline]
    pub fn new<T: Into<heapless::Vec<CharState, { Self::MAX_CHAR_STATES }>>>(char_states: T) -> Self {
        let v = char_states.into();
        Self {
            char_states: CapList::from_slice_copy(&v),
        }
    }

    #[inline]
    pub fn is_valid_char_idx(&self, char_idx: u8) -> bool {
        if char_idx < self.len() {
            return !self.char_states[char_idx].is_invalid();
        }
        false
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.char_states.is_empty()
    }

    #[inline]
    pub fn len(&self) -> u8 {
        self.char_states.len()
    }

    #[inline]
    pub fn iter_all(&self) -> impl Iterator<Item = &CharState> {
        self.char_states.slice().iter()
    }

    #[inline]
    pub fn iter_all_mut(&mut self) -> impl Iterator<Item = &mut CharState> {
        self.char_states.slice_mut().iter_mut()
    }

    #[inline]
    pub fn enumerate_valid(&self) -> impl Iterator<Item = (u8, &CharState)> {
        self.char_states
            .slice()
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.is_invalid())
            .map(|(i, v)| (i as u8, v))
    }

    #[inline]
    pub fn enumerate_valid_mut(&mut self) -> impl Iterator<Item = (u8, &mut CharState)> {
        self.char_states
            .slice_mut()
            .iter_mut()
            .enumerate()
            .filter(|(_, v)| !v.is_invalid())
            .map(|(i, v)| (i as u8, v))
    }

    #[inline]
    pub fn iter_valid(&self) -> impl Iterator<Item = &CharState> {
        self.char_states
            .slice()
            .iter()
            .enumerate()
            .filter(|(_, v)| !v.is_invalid())
            .map(|(_, v)| v)
    }

    #[inline]
    pub fn iter_valid_mut(&mut self) -> impl Iterator<Item = &mut CharState> {
        self.char_states
            .slice_mut()
            .iter_mut()
            .enumerate()
            .filter(|(_, v)| v.is_invalid())
            .map(|(_, v)| v)
    }
}

impl Index<u8> for CharStates {
    type Output = CharState;
    #[inline]
    fn index(&self, index: u8) -> &Self::Output {
        &self.char_states[index]
    }
}

impl IndexMut<u8> for CharStates {
    #[inline]
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.char_states[index]
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatusState {
    pub status_id: StatusId,
    pub eff_state: AppliedEffectState,
}

/// Return value of `StatusImpl` handlers to indicate that the handler had
/// some effect to the game state and a change needs to be applied to the
/// `AppliedEffectState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppliedEffectResult {
    /// Indicates the `StatusImpl` handler had effects
    /// but no changes will be applied to the `AppliedEffectState`.
    /// For implementing "Duration (Rounds)" and indefinite statuses.
    NoChange,
    DeleteSelf,
    ConsumeUsage,
    ConsumeUsages(u8),
    ConsumeOncePerRound,
    SetCounter(u8),
    SetCounterAndConsumeOncePerRound(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct SummonState {
    pub summon_id: SummonId,
    pub eff_state: AppliedEffectState,
}

pub type CharSelection = u8;

impl CharState {
    const HP_MASK: u8 = 0b0001_1111;
    const ENERGY_MASK: u8 = 0b1110_0000;
    const ENERGY_SHIFT: u8 = 5;

    #[inline]
    pub fn new(char_id: CharId) -> CharState {
        CharState {
            char_id,
            _hp_and_energy: char_id.char_card().max_health,
            total_dmg_taken: Default::default(),
            applied: Default::default(),
            flags: Default::default(),
            element_priority: Default::default(),
        }
    }

    #[inline]
    pub fn hp(&self) -> u8 {
        self._hp_and_energy & Self::HP_MASK
    }

    #[inline]
    pub fn set_hp(&mut self, hp: u8) {
        self._hp_and_energy = (self._hp_and_energy & !Self::HP_MASK) | (hp & Self::HP_MASK);
    }

    #[inline]
    pub fn energy(&self) -> u8 {
        (self._hp_and_energy & Self::ENERGY_MASK) >> Self::ENERGY_SHIFT
    }

    #[inline]
    pub fn set_energy(&mut self, energy: u8) {
        self._hp_and_energy = (self._hp_and_energy & !Self::ENERGY_MASK) | (energy << Self::ENERGY_SHIFT);
    }

    #[inline]
    pub fn is_max_hp(&self) -> bool {
        self.hp() >= self.char_id.char_card().max_health
    }

    #[inline]
    pub fn reduce_hp(&mut self, dmg_value: u8) {
        let h = self.hp();
        self.set_hp(h.saturating_sub(dmg_value));
    }

    #[inline]
    pub(crate) fn add_dmg_taken(&mut self, dmg_value: u8) {
        self.total_dmg_taken = self.total_dmg_taken.saturating_add(dmg_value);
    }

    #[inline]
    pub fn total_dmg_taken(&self) -> u8 {
        self.total_dmg_taken
    }

    pub(crate) fn skill_flags(&self, skill_id: SkillId) -> EnumSet<CharFlag> {
        let char_chrd = self.char_id.char_card();
        if let Some((i, _)) = char_chrd
            .skills
            .iter()
            .copied()
            .enumerate()
            .find(|(_, sid)| *sid == skill_id)
        {
            match i {
                0 => enum_set![CharFlag::SkillCastedThisTurn0],
                1 => enum_set![CharFlag::SkillCastedThisTurn1],
                2 => enum_set![CharFlag::SkillCastedThisTurn2],
                3 => enum_set![CharFlag::SkillCastedThisTurn3],
                _ => Default::default(),
            }
        } else {
            Default::default()
        }
    }

    #[inline]
    pub fn has_talent_equipped(&self) -> bool {
        self.flags.contains(CharFlag::TalentEquipped)
    }

    #[inline]
    pub fn char_id(&self) -> CharId {
        self.char_id
    }

    #[inline]
    pub fn char_card(&self) -> &'static CharCard {
        self.char_id.char_card()
    }

    #[inline]
    pub fn applied(&self) -> ElementSet {
        self.applied
    }

    #[inline]
    pub fn incremental_element_priority(&self) -> Option<ElementPriority> {
        self.element_priority
    }

    #[inline]
    pub fn set_incremental_element_priority(&mut self, ep: ElementPriority) {
        self.element_priority = Some(ep)
    }

    #[inline]
    pub fn clear_incremental_element_priority(&mut self) {
        self.element_priority = None;
    }
}
