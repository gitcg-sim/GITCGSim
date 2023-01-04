use super::*;
use crate::cards::all::*;
use crate::types::card_defs::Card;

impl GetCard for CardId {
    #[inline]
    fn get_card(self) -> &'static Card {
        crate::__generated_enum_cases!(CardId, self, &C)
    }
}
