use super::*;

pub const C: CharCard = CharCard {
    name: "Razor",
    elem: Element::Electro,
    weapon: WeaponType::Claymore,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![SkillId::SteelFang, SkillId::ClawAndThunder, SkillId::LightningFang],
    passive: None,
};

pub const STEEL_FANG: Skill = skill_na("Steel Fang", Element::Electro, 2, DealDMGType::Physical);

pub const CLAW_AND_THUNDER: Skill = Skill {
    name: "Claw and Thunder",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Electro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 3, 0)),
    ..Skill::new()
};

pub const LIGHTNING_FANG: Skill = Skill {
    name: "Lightning Fang",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Electro, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 5, 0)),
    apply: Some(StatusId::TheWolfWithin),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::SteelFang, STEEL_FANG),
    (SkillId::ClawAndThunder, CLAW_AND_THUNDER),
    (SkillId::LightningFang, LIGHTNING_FANG),
];

pub mod the_wolf_within {
    use super::*;
    pub const S: Status = Status::new_duration("The Wolf Within", StatusAttachMode::Character, 2);

    decl_event_handler_trait_impl!(OwnCharacterSkillEvent(TheWolfWithin), I);
    impl OwnCharacterSkillEvent for TheWolfWithin {
        const SKILL_TYPES: EnumSet<SkillType> = enum_set![SkillType::NormalAttack | SkillType::ElementalSkill];

        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            e.cmd_deal_dmg(DealDMGType::ELECTRO, 2, 0);
            Some(AppliedEffectResult::NoChange)
        }
    }
}
