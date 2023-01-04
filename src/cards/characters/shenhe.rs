use super::*;

pub const C: CharCard = CharCard {
    name: "Shenhe",
    elem: Element::Cryo,
    weapon: WeaponType::Polearm,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::DawnstarPiercer,
        SkillId::SpringSpiritSummoning,
        SkillId::DivineMaidensDeliverance,
    ],
    passive: None,
};

pub const DAWNSTAR_PIERCER: Skill = skill_na("Dawnstar Piercer", Element::Cryo, 2, DealDMGType::Physical);

pub const SPRING_SPIRIT_SUMMONING: Skill = Skill {
    name: "Spring Spirit Summoning",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Cryo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 2, 0)),
    apply: Some(StatusId::IcyQuill),
    ..Skill::new()
};

pub const DIVINE_MAIDENS_DELIVERANCE: Skill = Skill {
    name: "Divine Maiden's Deliverance",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Cryo, 3, 0, 2),
    summon: Some(SummonSpec::One(SummonId::TalismanSpirit)),
    ..Skill::new()
};

pub mod icy_quill {
    use super::*;

    pub const S: Status = Status::new_usages("Icy Quill", StatusAttachMode::Team, 3, None);

    decl_status_impl_type!(IcyQuill, I);
    impl StatusImpl for IcyQuill {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::LateOutgoingDMG]
        }

        fn late_outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(Element::Cryo) = dmg.dmg_type.element() else { return None };
            dmg.dmg += 1;
            if e.has_talent_equipped()
                && e.skill_type() == Some(SkillType::NormalAttack)
                && e.eff_state.can_use_once_per_round()
            {
                Some(AppliedEffectResult::ConsumeOncePerRound)
            } else {
                Some(AppliedEffectResult::ConsumeUsage)
            }
        }
    }
}

pub mod talisman_spirit {
    use super::*;

    pub const S: Status = Status::new_usages("Talisman Spirit", StatusAttachMode::Character, 2, None);

    struct TalismanSpiritIncreaseDMG();
    impl StatusImpl for TalismanSpiritIncreaseDMG {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::LateOutgoingDMG]
        }

        fn late_outgoing_dmg(&self, _: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let (DealDMGType::Physical | DealDMGType::Elemental(Element::Cryo)) = dmg.dmg_type else { return None };
            dmg.dmg += 1;
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }

    decl_summon_impl_type!(TalismanSpirit, I);
    compose_status_impls!(TalismanSpirit(
        EndPhaseDealDMG(deal_elem_dmg(Element::Cryo, 1, 0)),
        TalismanSpiritIncreaseDMG(),
    ));
}
