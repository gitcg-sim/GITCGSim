use super::*;

pub const C: CharCard = CharCard {
    name: "Ganyu",
    elem: Element::Cryo,
    weapon: WeaponType::Bow,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::LiutianArchery,
        SkillId::TrailOfTheQilin,
        SkillId::FrostflakeArrow,
        SkillId::CelestialShower,
    ],
    passive: None,
};

pub const LIUTIAN_ARCHERY: Skill = skill_na("Liutain Archery", Element::Cryo, 2, DealDMGType::Physical);

pub const TRAIL_OF_THE_QILIN: Skill = Skill {
    name: "Trail of the Qilin",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Cryo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 1, 0)),
    apply: Some(StatusId::IceLotus),
    ..Skill::new()
};

pub const FROSTFLAKE_ARROW: Skill = Skill {
    name: "Frostflake Arrow",
    skill_type: SkillType::NormalAttack,
    cost: cost_elem(Element::Cryo, 5, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 2, 2)),
    apply: Some(StatusId::IceLotus),
    ..Skill::new()
};

pub const CELESTIAL_SHOWER: Skill = Skill {
    name: "Celestial Shower",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Cryo, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::SacredCryoPearl)),
    ..Skill::new()
};

pub mod ice_lotus {
    use super::*;

    pub const S: Status = Status::new_usages("Ice Lotus", StatusAttachMode::Team, 2, None);

    decl_status_impl_type!(IceLotus, I);
    impl StatusImpl for IceLotus {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG]
        }

        fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.try_reduce(1, AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod sacred_cryo_pearl {
    use super::*;

    pub const S: Status = Status::new_summon_usages("Sacred Cryo Pearl", 2);

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(DealDMG::new(DealDMGType::CRYO, 1, 0));
}
