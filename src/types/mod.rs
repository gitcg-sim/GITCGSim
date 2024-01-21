pub mod by_player;

pub mod card_defs;

// TODO refactor out forwarding imports
pub use crate::tcg_model;

pub mod card_selection;

pub mod applied_effect_state;
pub(crate) mod card_impl;
pub(crate) mod command;
pub use crate::dice_counter;
pub mod char_state;
pub mod game_state;
pub mod input;
pub mod logging;
pub mod nondet;
pub mod status_collection;
mod status_spec_modifier;
pub use status_spec_modifier::*;
pub(crate) mod status_impl;

pub type ElementSet = enumset::EnumSet<crate::tcg_model::Element>;

#[macro_export]
macro_rules! elem_set {
    () => {
        {
            let es: enumset::EnumSet::<$crate::tcg_model::Element> = enumset::EnumSet::new();
            es
        }
    };
    ($($x: expr),+ $(,)?) => {
        {
            let mut es: enumset::EnumSet::<$crate::tcg_model::Element> = enumset::EnumSet::new();
            $(
                es.insert($x);
            )+
            es
        }
    };
}
