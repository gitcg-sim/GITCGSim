use super::*;

pub const C: CharCard = CharCard {
    name: "Sangonomiya Kokomi",
    elem: Element::Hydro,
    weapon: WeaponType::Catalyst,
    faction: Faction::Inazuma,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::TheShapeOfWater,
        SkillId::KuragesOath,
        SkillId::NereidsAscension,
    ],
    passive: None,
};

pub const THE_SHAPE_OF_WATER: Skill = skill_na("The Shape of Water", Element::Hydro, 1, DealDMGType::HYDRO);

pub const KURAGES_OATH: Skill = Skill {
    name: "Kurage's Oath",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    summon: Some(SummonSpec::One(SummonId::BakeKurage)),
    commands: list8![Command::ApplyElementToSelf(Element::Hydro),],
    ..Skill::new()
};

pub const NEREIDS_ASCENSION: Skill = Skill {
    name: "Nereid's Ascension",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 3, 0)),
    apply: Some(StatusId::CeremonialGarment),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::TheShapeOfWater, THE_SHAPE_OF_WATER),
    (SkillId::KuragesOath, KURAGES_OATH),
    (SkillId::NereidsAscension, NEREIDS_ASCENSION),
];

pub mod bake_kurage {
    use super::*;

    pub const S: Status = Status::new_usages("Bake-Kurage", StatusAttachMode::Summon, 2, None);

    pub const I: EndPhaseCommands = EndPhaseCommands(list8![
        Command::DealDMG(deal_elem_dmg(Element::Hydro, 1, 0)),
        Command::Heal(1, CmdCharIdx::Active),
    ]);
}

pub mod ceremonial_garment {
    use super::*;

    pub const S: Status = Status::new_duration("Ceremonial Garment", StatusAttachMode::Character, 2);

    decl_status_impl_type!(CeremonialGarment, I);
    impl StatusImpl for CeremonialGarment {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_NA
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillType::NormalAttack) = e.skill_type() else {
                return None;
            };
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let SkillType::NormalAttack = e.event_skill_ensuring_attached_character()?.skill_type() else {
                return None;
            };
            e.out_cmds.push((*e.ctx_for_dmg, Command::HealAll(1)));
            Some(AppliedEffectResult::NoChange)
        }
    }
}
