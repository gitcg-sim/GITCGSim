pub mod by_player;

pub mod card_defs;

// TODO refactor out forwarding imports
pub mod enums {
    pub use crate::tcg_model::enums::*;
}

// TODO refactor out forwarding imports
pub mod deal_dmg {
    pub use crate::tcg_model::deal_dmg::*;
}

pub mod card_selection;

pub mod applied_effect_state;
pub(crate) mod card_impl;
pub(crate) mod command;
pub mod dice_counter {
    pub use crate::dice_counter::*;
}
pub mod char_state;
pub mod game_state;
pub mod input;
pub mod logging;
pub mod nondet;
mod status_spec_modifier;
pub use status_spec_modifier::*;
pub(crate) mod status_impl;

pub type ElementSet = enumset::EnumSet<crate::tcg_model::enums::Element>;

#[macro_export]
macro_rules! elem_set {
    () => {
        {
            let es: enumset::EnumSet::<$crate::tcg_model::enums::Element> = enumset::EnumSet::new();
            es
        }
    };
    ($($x: expr),+ $(,)?) => {
        {
            let mut es: enumset::EnumSet::<$crate::tcg_model::enums::Element> = enumset::EnumSet::new();
            $(
                es.insert($x);
            )+
            es
        }
    };
}
