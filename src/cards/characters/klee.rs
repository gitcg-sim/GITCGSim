use super::*;

pub const C: CharCard = CharCard {
    name: "Klee",
    elem: Element::Pyro,
    weapon: WeaponType::Catalyst,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![SkillId::Kaboom, SkillId::JumpyDumpty, SkillId::SparksNSplash],
    passive: None,
};

pub const KABOOM: Skill = skill_na("Kaboom!", Element::Pyro, 1, DealDMGType::PYRO);

pub const JUMPY_DUMPTY: Skill = Skill {
    name: "Jumpy Dumpty",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Pyro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 3, 0)),
    apply: Some(StatusId::ExplosiveSpark),
    ..Skill::new()
};

pub const SPARKS_N_SPLASH: Skill = Skill {
    name: "Sparks 'n' Splash",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Pyro, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 3, 0)),
    commands: list8![Command::ApplyTeamStatusToTargetPlayer(StatusId::SparksNSplash)],
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::Kaboom, KABOOM),
    (SkillId::JumpyDumpty, JUMPY_DUMPTY),
    (SkillId::SparksNSplash, SPARKS_N_SPLASH),
];

pub mod sparks_n_splash {
    use super::*;

    pub const S: Status =
        Status::new_usages("Sparks 'n' Splash", StatusAttachMode::Team, 2, None).applies_to_opposing();

    decl_status_impl_type!(SparksNSplash, I);
    impl StatusImpl for SparksNSplash {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let _ = e.event_skill_ensuring_own_player()?;
            e.out_cmds.push((
                e.ctx_for_dmg.without_target(),
                Command::TakeDMG(DealDMG::new(DealDMGType::PYRO, 2, 0)),
            ));
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod explosive_spark {
    use super::*;

    pub const S: Status = Status::new_usages("Explosive Spark", StatusAttachMode::Character, 1, None)
        .talent_usages_increase(CharId::Klee, 1);

    decl_status_impl_type!(ExplosiveSpark, I);
    impl StatusImpl for ExplosiveSpark {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::UpdateCost]
        }

        fn update_cost(
            &self,
            e: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            if !e.is_charged_attack() {
                return None;
            }

            let CostType::Skill(SkillId::Kaboom) = cost_type else {
                return None;
            };

            cost.try_reduce_elemental_cost(1, Element::Pyro)
                .then_some(AppliedEffectResult::NoChange)
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !e.is_charged_attack() {
                return None;
            }

            dmg.dmg += 1;
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
