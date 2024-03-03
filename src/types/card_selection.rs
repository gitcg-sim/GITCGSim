use smallvec::SmallVec;

use crate::cards::ids::SummonId;

use super::{
    by_player::ByPlayer,
    game_state::{PlayerId, PlayerState},
    status_collection::StatusCollection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CardSelectionSpec {
    OwnCharacter,
    OwnSummon,
    OpponentSummon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CardSelection {
    OwnCharacter(u8),
    OwnSummon(SummonId),
    OpponentSummon(SummonId),
}

impl CardSelectionSpec {
    #[inline]
    pub fn validate(self, sel: Option<CardSelection>) -> bool {
        match self {
            Self::OwnCharacter => matches!(sel, Some(CardSelection::OwnCharacter(..))),
            Self::OwnSummon => matches!(sel, Some(CardSelection::OwnSummon(..))),
            Self::OpponentSummon => matches!(sel, Some(CardSelection::OpponentSummon(..))),
        }
    }

    #[inline]
    pub(crate) fn validate_selection(
        self,
        sel: CardSelection,
        players: &ByPlayer<PlayerState>,
        status_collections: &ByPlayer<StatusCollection>,
        player_id: PlayerId,
    ) -> bool {
        match (self, sel) {
            (Self::OwnCharacter, CardSelection::OwnCharacter(ci)) => players.get(player_id).is_valid_char_idx(ci),
            (Self::OwnSummon, CardSelection::OwnSummon(summon_id)) => {
                status_collections.get(player_id).has_summon(summon_id)
            }
            (Self::OpponentSummon, CardSelection::OpponentSummon(summon_id)) => {
                status_collections.get(player_id.opposite()).has_summon(summon_id)
            }
            (_, _) => false,
        }
    }

    #[inline]
    pub(crate) fn available_selections(
        self,
        players: &ByPlayer<PlayerState>,
        status_collections: &ByPlayer<StatusCollection>,
        player_id: PlayerId,
    ) -> SmallVec<[CardSelection; 4]> {
        match self {
            Self::OwnCharacter => players
                .get(player_id)
                .char_states
                .enumerate_valid()
                .map(|(i, _)| CardSelection::OwnCharacter(i))
                .collect(),
            Self::OwnSummon => status_collections
                .get(player_id)
                .iter_entries()
                .filter_map(|entry| entry.key.summon_id().map(CardSelection::OwnSummon))
                .collect(),
            Self::OpponentSummon => status_collections
                .get(player_id)
                .iter_entries()
                .filter_map(|entry| entry.key.summon_id().map(CardSelection::OpponentSummon))
                .collect(),
        }
    }
}
