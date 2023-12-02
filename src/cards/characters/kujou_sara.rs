use super::*;

pub const C: CharCard = CharCard {
    name: "Kujou Sara",
    elem: Element::Electro,
    weapon: WeaponType::Bow,
    faction: Faction::Inazuma,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::TenguBowmanship,
        SkillId::TenguStormcall,
        SkillId::SubjugationKoukouSendou,
    ],
    passive: None,
};

pub const TENGU_BOWMANSHIP: Skill = skill_na("Tengu Bowmanship", Element::Electro, 2, DealDMGType::Physical);

pub const TENGU_STORMCALL: Skill = Skill {
    name: "Tengu Stormcall",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Electro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::TenguJuuraiAmbush)),
    ..Skill::new()
};

pub const SUBJUGATION_KOUKOU_SENDOU: Skill = Skill {
    name: "Subjugation: Koukou Sendou",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Electro, 4, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::TenguJuuraiStormcluster)),
    ..Skill::new()
};

pub mod tengu_juurai_ambush {
    use super::*;

    pub const S: Status = Status::new_summon_usages("Tengu Juurai: Ambush", 1);

    pub const I: EndPhaseCommands = EndPhaseCommands(list8![
        Command::DealDMG(deal_elem_dmg(Element::Electro, 2, 0)),
        Command::ApplyCharacterStatus(StatusId::CrowfeatherCover, CmdCharIdx::Active),
    ]);
}

pub mod tengu_juurai_stormcluster {
    use super::*;

    pub const S: Status = Status::new_summon_usages("Tengu Juurai: Stormcluster", 2);

    pub const I: EndPhaseCommands = EndPhaseCommands(list8![
        Command::DealDMG(deal_elem_dmg(Element::Electro, 2, 0)),
        Command::ApplyCharacterStatus(StatusId::CrowfeatherCover, CmdCharIdx::Active),
    ]);
}

pub mod crowfeather_cover {
    use super::*;

    pub const S: Status = Status::new_usages("Crowfeather Cover", StatusAttachMode::Character, 1, None);

    decl_status_impl_type!(CrowfeatherCover, I);
    impl StatusImpl for CrowfeatherCover {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillType::ElementalSkill | SkillType::ElementalBurst) = e.skill_type() else {
                return None;
            };
            dmg.dmg += 1;
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
