use crate::types::{
    card_defs::{Cost, Skill},
    enums::{Element, SkillType},
};

use crate::types::deal_dmg::{DealDMG, DealDMGType};

pub const fn cost_elem(elem: Element, elem_cost: u8, unaligned_cost: u8, energy_cost: u8) -> Cost {
    Cost {
        elem_cost: Some((elem, elem_cost)),
        unaligned_cost,
        aligned_cost: 0,
        energy_cost,
    }
}

pub const fn cost(elem_cost: Option<(Element, u8)>, unaligned_cost: u8, aligned_cost: u8, energy_cost: u8) -> Cost {
    Cost {
        elem_cost,
        unaligned_cost,
        aligned_cost,
        energy_cost,
    }
}

pub const fn deal_elem_dmg(elem: Element, dmg: u8, piercing_dmg_to_standby: u8) -> DealDMG {
    DealDMG {
        dmg_type: DealDMGType::Elemental(elem),
        dmg,
        piercing_dmg_to_standby,
    }
}

pub const fn skill_na(name: &'static str, elem: Element, dmg: u8, dmg_type: DealDMGType) -> Skill {
    Skill {
        name,
        skill_type: SkillType::NormalAttack,
        cost: cost(Some((elem, 1)), 2, 0, 0),
        deal_dmg: Some(DealDMG {
            dmg_type,
            dmg,
            piercing_dmg_to_standby: 0,
        }),
        ..Skill::new()
    }
}
