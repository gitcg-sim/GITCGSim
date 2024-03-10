use crate::std_subset::Vec;
use crate::{cards::ids::CharId, status_impls::prelude::Element};

use super::{CharFlag, CharState};

crate::with_updaters!(
    #[derive(Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct CharStateBuilder {
        pub char_id: CharId,
        pub hp: u8,
        pub energy: u8,

        #[cfg_attr(feature = "serde", serde(default))]
        pub applied: Vec<Element>,
        #[cfg_attr(feature = "serde", serde(default))]
        pub flags: Vec<CharFlag>,
        #[cfg_attr(feature = "serde", serde(default))]
        pub total_dmg_taken: u8,
    }
);

impl CharStateBuilder {
    pub fn new(char_id: CharId) -> Self {
        Self {
            char_id,
            hp: Default::default(),
            energy: Default::default(),
            applied: Default::default(),
            flags: Default::default(),
            total_dmg_taken: Default::default(),
        }
    }

    pub fn build(&self) -> CharState {
        let mut cs = CharState {
            char_id: self.char_id,
            _hp_and_energy: Default::default(),
            applied: self.applied.iter().copied().collect(),
            flags: self.flags.iter().copied().collect(),
            total_dmg_taken: self.total_dmg_taken,
            element_priority: Default::default(),
        };
        cs.set_hp(self.hp);
        cs.set_energy(self.energy);
        cs
    }
}

impl CharState {
    pub fn into_builder(&self) -> CharStateBuilder {
        CharStateBuilder {
            char_id: self.char_id,
            hp: self.hp(),
            energy: self.energy(),
            applied: self.applied.iter().collect(),
            flags: self.flags.iter().collect(),
            total_dmg_taken: self.total_dmg_taken,
        }
    }
}

crate::impl_from_to_builder!(CharState, CharStateBuilder);
