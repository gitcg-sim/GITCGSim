use super::*;

pub const C: CharCard = CharCard {
    name: "Zhongli",
    elem: Element::Geo,
    weapon: WeaponType::Polearm,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::RainOfStone,
        SkillId::DominusLapidis,
        SkillId::DominusLapidisStrikingStone,
        SkillId::PlanetBefall,
    ],
    passive: None,
};

pub const RAIN_OF_STONE: Skill = skill_na("Rain of Stone", Element::Geo, 2, DealDMGType::Physical);

pub const DOMINUS_LAPIDIS: Skill = Skill {
    name: "Dominus Lapidis",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Geo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::StoneStele)),
    ..Skill::new()
};

pub const DOMINUS_LAPIDIS_STRIKING_STONE: Skill = Skill {
    name: "Dominus Lapidis: Striking Stone",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Geo, 5, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 3, 0)),
    summon: Some(SummonSpec::One(SummonId::StoneStele)),
    apply: Some(StatusId::JadeShield),
    ..Skill::new()
};

pub const PLANET_BEFALL: Skill = Skill {
    name: "Planet Befall",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Geo, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 4, 0)),
    commands: list8![Command::ApplyCharacterStatusToTarget(StatusId::Petrification),],
    ..Skill::new()
};

pub mod stone_stele {
    use super::*;

    pub const S: Status = Status::new_usages("Stone Stele", StatusAttachMode::Summon, 2, None);

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(deal_elem_dmg(Element::Geo, 1, 0));
}

pub mod jade_shield {
    use super::*;

    pub const S: Status = Status::new_shield_points("Jade Shield", StatusAttachMode::Character, 2, None);

    decl_status_impl_type!(JadeShield, I);
    impl StatusImpl for JadeShield {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![]
        }
    }
}

pub mod petrification {
    use super::*;

    pub const S: Status = Status::new_duration("Petrification", StatusAttachMode::Character, 1).applies_to_opposing();

    decl_status_impl_type!(Petrification, I);
    impl StatusImpl for Petrification {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::CannotPerformActions]
        }
    }
}
