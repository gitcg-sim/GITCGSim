use crate::cards::ids::*;
use crate::data_structures::CommandList;
use crate::status_impls::primitives::all::*;
use crate::tcg_model::deal_dmg::*;
use crate::types::{card_defs::*, command::*, enums::*, game_state::*, status_impl::*};
use crate::types::{card_impl::*, dice_counter::DiceCounter};
use crate::{decl_status_impl_type, list8};
use enumset::{enum_set, EnumSet};

pub struct DefaultCardImpl();
impl CardImpl for DefaultCardImpl {}

pub mod blank_card;

pub mod the_bestest_travel_companion {
    use super::*;

    pub const C: Card = Card {
        name: "The Bestest Travel Companion!",
        cost: Cost::unaligned(2),
        effects: list8![Command::AddDice(DiceCounter::omni(2))],
        card_type: CardType::Event,
        card_impl: None,
    };
}

pub mod starsigns {
    use super::*;

    pub const C: Card = Card {
        name: "Starsigns",
        cost: Cost::unaligned(2),
        effects: list8![Command::AddEnergy(1, CmdCharIdx::Active)],
        card_type: CardType::Event,
        card_impl: None,
    };
}

pub mod strategize {
    use super::*;

    pub const C: Card = Card {
        name: "Strategize",
        cost: Cost::ONE,
        effects: list8![Command::DrawCards(2, None)],
        card_type: CardType::Event,
        card_impl: None,
    };
}

pub mod leave_it_to_me;

pub mod changing_shifts;

pub mod lightning_stiletto;

pub mod i_havent_lost_yet;

pub mod when_the_crane_returned;

pub mod quick_knit;

pub mod send_off;

pub mod calxs_arts;

pub mod food;
pub use food::*;

pub mod elemental_resonance;
pub use elemental_resonance::*;
