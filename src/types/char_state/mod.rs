#![allow(non_snake_case)]
use std::fmt::Debug;

use enumset::{enum_set, EnumSet, EnumSetType};

use crate::cards::ids::*;

pub use crate::types::applied_effect_state::AppliedEffectState;
use crate::types::ElementSet;

#[derive(Debug, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[enumset(repr = "u8")]
pub enum CharFlag {
    TalentEquipped,
    SkillCastedThisTurn0,
    SkillCastedThisTurn1,
    SkillCastedThisTurn2,
    SkillCastedThisTurn3,
}

impl CharFlag {
    pub const RETAIN: EnumSet<Self> = enum_set![Self::TalentEquipped];
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharState {
    pub char_id: CharId,
    _hp_and_energy: u8,
    pub applied: ElementSet,
    pub flags: EnumSet<CharFlag>,
}

impl Debug for CharState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharState")
            .field("char_id", &self.char_id)
            .field("hp", &self.get_hp())
            .field("energy", &self.get_energy())
            .field("applied", &self.applied)
            .field("flags", &self.flags)
            .finish()
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
            _hp_and_energy: char_id.get_char_card().max_health,
            applied: Default::default(),
            flags: Default::default(),
        }
    }

    #[inline]
    pub fn get_hp(&self) -> u8 {
        self._hp_and_energy & Self::HP_MASK
    }

    #[inline]
    pub fn set_hp(&mut self, hp: u8) {
        self._hp_and_energy = (self._hp_and_energy & !Self::HP_MASK) | (hp & Self::HP_MASK);
    }

    #[inline]
    pub fn get_energy(&self) -> u8 {
        (self._hp_and_energy & Self::ENERGY_MASK) >> Self::ENERGY_SHIFT
    }

    #[inline]
    pub fn set_energy(&mut self, energy: u8) {
        self._hp_and_energy = (self._hp_and_energy & !Self::ENERGY_MASK) | (energy << Self::ENERGY_SHIFT);
    }

    #[inline]
    pub fn reduce_hp(&mut self, dmg_value: u8) {
        let h = self.get_hp();
        if dmg_value > h {
            self.set_hp(0);
        } else {
            self.set_hp(h - dmg_value);
        }
    }

    pub(crate) fn skill_flags(&self, skill_id: SkillId) -> EnumSet<CharFlag> {
        let char_chrd = self.char_id.get_char_card();
        if let Some((i, _)) = char_chrd
            .skills
            .to_vec()
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
}
