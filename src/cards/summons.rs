use crate::status_impls::primitives::all::*;
use crate::tcg_model::deal_dmg::{DealDMG, DealDMGType};
use crate::types::{card_defs::Status, enums::StatusAttachMode};

pub mod burning_flame {
    use super::*;

    pub const S: Status = Status::new_usages("Burning Flame", StatusAttachMode::Summon, 1, Some(2));

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(DealDMG::new(DealDMGType::PYRO, 1, 0));
}
