pub mod ids;

pub mod builders;

pub mod characters;

pub mod skills;

pub mod summons;

pub mod statuses;

pub mod support;

pub mod event;

pub mod equipment;

pub(crate) mod char_reexports {
    pub use crate::ids::__generated_char_reexports::*;
}

/// Re-exports for all `Card`-related modules.
/// Used by auto-generate code to lookup card implementations.
pub mod all_cards_reexports {
    pub use super::equipment::{artifact::*, talent::*, weapon::*};
    pub use super::event::*;
    pub use super::support::*;
}
