use super::*;

pub const C: CharCard = CharCard {
    name: "Fischl",
    elem: Element::Electro,
    weapon: WeaponType::Bow,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::BoltsOfDownfall,
        SkillId::Nightrider,
        SkillId::MidnightPhantasmagoria,
    ],
    passive: None,
};

pub const BOLTS_OF_DOWNFALL: Skill = skill_na("Bolts of Downfall", Element::Electro, 2, DealDMGType::Physical);

pub const NIGHTRIDER: Skill = Skill {
    name: "Nightrider",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Electro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::Oz)),
    ..Skill::new()
};

pub const MIDNIGHT_PHANTASMAGORIA: Skill = Skill {
    name: "Midnight Phantasmagoria",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Electro, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 4, 2)),
    ..Skill::new()
};

pub mod oz {
    use super::*;

    pub const S: Status = Status::new_summon_usages("Oz", 2);

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(DealDMG::new(DealDMGType::ELECTRO, 1, 0));
}
