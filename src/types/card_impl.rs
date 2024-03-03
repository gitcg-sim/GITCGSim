use crate::{cards::ids::CardId, data_structures::CommandList, dispatcher_ops::DispatchError};

use crate::types::{
    card_defs::Card,
    command::{Command, CommandContext},
    game_state::*,
};

use super::by_player::ByPlayer;

pub struct CardImplContext<'a> {
    pub players: &'a ByPlayer<PlayerState>,
    pub status_collections: ByPlayer<&'a StatusCollection>,
    pub active_player_id: PlayerId,
    pub card_id: CardId,
    pub card: &'static Card,
    pub selection: Option<CardSelection>,
}

impl<'a> CardImplContext<'a> {
    #[inline]
    pub fn get_next_available_suport_slot(&self) -> Option<SupportSlot> {
        self.status_collections[self.active_player_id].get_next_available_support_slot()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CanBePlayedResult {
    CanBePlayed,
    CannotBePlayed,
    InvalidSelection,
}

impl CanBePlayedResult {
    #[inline]
    pub fn to_bool(self) -> bool {
        matches!(self, CanBePlayedResult::CanBePlayed)
    }

    #[inline]
    pub fn to_result(self) -> Result<(), DispatchError> {
        match self {
            CanBePlayedResult::CanBePlayed => Ok(()),
            CanBePlayedResult::InvalidSelection => Err(DispatchError::InvalidSelection),
            CanBePlayedResult::CannotBePlayed => Err(DispatchError::UnableToPlayCard),
        }
    }
}

/// Trait for programmatic implementation of a non-character card:
///  - Equipment (Weapon/Artifact/Talent)
///  - Support
///  - Food
///  - Event
#[allow(unused_variables)]
pub trait CardImpl {
    /// Called to check if the card can be played at the current game state.
    /// The implementation must check if character indices in the selection are valid
    /// if this card takes character selections.
    fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
        CanBePlayedResult::CanBePlayed
    }

    /// Called to determine the kind of the selection this card requires.
    fn selection(&self) -> Option<CardSelectionSpec> {
        None
    }

    /// Precondition: The card can be played under the current game state and targeting.
    /// Called to determine the effects of the card.
    /// The default implementation adds commands based on the `body` field.
    fn get_effects(
        &self,
        cic: &CardImplContext,
        ctx: &CommandContext,
        commands: &mut CommandList<(CommandContext, Command)>,
    ) {
        for eff in cic.card.effects.to_vec_copy() {
            commands.push((*ctx, eff))
        }
    }
}
