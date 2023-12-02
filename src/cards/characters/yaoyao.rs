use super::*;

pub const C: CharCard = CharCard {
    name: "Yaoyao",
    elem: Element::Dendro,
    weapon: WeaponType::Polearm,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::TossNTurnSpear,
        SkillId::RaphanusSkyCluster,
        SkillId::MoonjadeDescent,
    ],
    passive: None,
};

pub const TOSS_N_TURN_SPEAR: Skill = skill_na("Toss 'N' Turn Spear", Element::Dendro, 2, DealDMGType::Physical);

pub const RAPHANUS_SKY_CLUSTER: Skill = Skill {
    name: "Raphanus Sky Cluster",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Dendro, 3, 0, 0),
    summon: Some(SummonSpec::One(SummonId::YueguiThrowingMode)),
    ..Skill::new()
};

pub const MOONJADE_DESCENT: Skill = Skill {
    name: "Moonjade Descent",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Dendro, 4, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 1, 0)),
    apply: Some(StatusId::AdeptalLegacy),
    ..Skill::new()
};

pub mod yuegui_throwing_mode {
    use super::*;

    pub const S: Status = Status::new_usages("Yuegui: Throwing Mode", StatusAttachMode::Summon, 2, None)
        .casted_by_character(CharId::Yaoyao);

    decl_summon_impl_type!(YueguiThrowingMode, I);
    trigger_event_impl!(YueguiThrowingMode, [Switched], |e| {
        if e.c.has_talent_equipped() && e.c.eff_state.get_usages() == 1 {
            e.cmd_deal_dmg(DealDMGType::DENDRO, 2, 0);
            e.add_cmd(Command::HealTakenMostDMG(2));
        } else {
            e.cmd_deal_dmg(DealDMGType::DENDRO, 1, 0);
            e.add_cmd(Command::HealTakenMostDMG(1));
        }
        Some(AppliedEffectResult::ConsumeUsage)
    });
}

pub mod adeptal_legacy {
    use super::*;

    pub const S: Status = Status::new_usages("Adeptal Legacy", StatusAttachMode::Character, 3, None);

    decl_status_impl_type!(AdeptalLegacy, I);
    trigger_event_impl!(AdeptalLegacy, [Switched], |e| {
        e.cmd_deal_dmg(DealDMGType::DENDRO, 1, 0);
        e.add_cmd(Command::Heal(1, CmdCharIdx::Active));
        Some(AppliedEffectResult::ConsumeUsage)
    });
}
