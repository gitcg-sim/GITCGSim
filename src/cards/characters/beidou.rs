use super::*;

pub const C: CharCard = CharCard {
    name: "Beidou",
    elem: Element::Electro,
    weapon: WeaponType::Claymore,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 3,
    skills: list8![SkillId::Oceanborne, SkillId::Tidecaller, SkillId::Stormbreaker,],
    passive: None,
};

pub const OCEANBORNE: Skill = skill_na("Oceanborne", Element::Electro, 2, DealDMGType::Physical);

pub const TIDECALLER: Skill = Skill {
    name: "Tidecaller",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Electro, 3, 0, 0),
    apply: Some(StatusId::TidecallerSurfEmbrace),
    ..Skill::new()
};

pub const STORMBREAKER: Skill = Skill {
    name: "Stormbreaker",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Electro, 4, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 3, 0)),
    apply: Some(StatusId::ThunderbeastsTarge),
    ..Skill::new()
};

pub const WAVESTRIDER: Skill = Skill {
    name: "Wavestrider",
    skill_type: SkillType::ElementalSkill,
    cost: Cost::ZERO,
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 2, 0)),
    ..Skill::new()
};

pub mod tidecaller_surf_embrace {
    use super::*;

    pub const S: Status = Status::new_shield_points("Tidecaller: Surf Embrace", StatusAttachMode::Character, 2, None)
        .with_prepare_skill(1);

    pub const I: PreparedSkill = PreparedSkill::new(SkillId::Wavestrider);
}

pub mod thunderbeasts_targe {
    use super::*;

    pub const S: Status = Status::new_duration("Thunderbeast's Targe", StatusAttachMode::Character, 2);

    struct ThunderbeastsTargeNA();
    impl OwnCharacterSkillEvent for ThunderbeastsTargeNA {
        const SKILL_TYPES: EnumSet<SkillType> = enum_set![SkillType::NormalAttack];
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            e.cmd_deal_dmg(DealDMGType::ELECTRO, 1, 0);
            Some(AppliedEffectResult::NoChange)
        }
    }

    decl_status_impl_type!(ThunderbeastsTarge, I);
    compose_status_impls!(ThunderbeastsTarge(
        ReduceDMGAbove::new(1, Some(3), AppliedEffectResult::NoChange),
        OwnCharacterSkillEventI(ThunderbeastsTargeNA()),
    ));
}
