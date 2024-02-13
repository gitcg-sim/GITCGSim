use super::*;

pub const C: CharCard = CharCard {
    name: "Diona",
    elem: Element::Cryo,
    weapon: WeaponType::Bow,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![SkillId::KatzleinStyle, SkillId::IcyPaws, SkillId::SignatureMix,],
    passive: None,
};

pub const KATZLEIN_STYLE: Skill = skill_na("Kätzlein Style", Element::Cryo, 2, DealDMGType::Physical);

pub const ICY_PAWS: Skill = Skill {
    name: "Icy Paws",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Cryo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 2, 0)),
    apply: Some(StatusId::CatClawShield),
    ..Skill::new()
};

pub const SIGNATURE_MIX: Skill = Skill {
    name: "Signature Mix",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Cryo, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::DrunkenMist)),
    commands: list8![Command::Heal(2, CmdCharIdx::Active),],
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::KatzleinStyle, KATZLEIN_STYLE),
    (SkillId::IcyPaws, ICY_PAWS),
    (SkillId::SignatureMix, SIGNATURE_MIX),
];

pub mod cat_claw_shield {
    use super::*;

    pub const S: Status = Status::new_shield_points("Cat-Claw Shield", StatusAttachMode::Team, 1, None)
        .talent_usages_increase(CharId::Diona, 1);

    pub const I: EmptyStatusImpl = EmptyStatusImpl();
}

pub mod drunken_mist {
    use super::*;

    pub const S: Status = Status::new_usages("Drunken Mist", StatusAttachMode::Summon, 2, None);

    pub const I: EndPhaseCommands = EndPhaseCommands(list8![
        Command::DealDMG(DealDMG::new(DealDMGType::CRYO, 1, 0)),
        Command::Heal(2, CmdCharIdx::Active)
    ]);
}
