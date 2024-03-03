use super::*;

pub const C: CharCard = CharCard {
    name: "Venti",
    elem: Element::Anemo,
    weapon: WeaponType::Bow,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::DivineMarksmanship,
        SkillId::SkywardSonnet,
        SkillId::WindsGrandOde,
    ],
    passive: None,
};

pub const DIVINE_MARKSMANSHIP: Skill = skill_na("Divine Marksmanship", Element::Anemo, 2, DealDMGType::Physical);

pub const SKYWARD_SONNET: Skill = Skill {
    name: "Skyward Sonnet",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Anemo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Anemo, 2, 0)),
    apply: Some(StatusId::Stormzone),
    ..Skill::new()
};

pub const WINDS_GRAND_ODE: Skill = Skill {
    name: "Wind's Grand Ode",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Anemo, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Anemo, 2, 0)),
    summon: Some(SummonSpec::One(SummonId::Stormeye)),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::DivineMarksmanship, DIVINE_MARKSMANSHIP),
    (SkillId::SkywardSonnet, SKYWARD_SONNET),
    (SkillId::WindsGrandOde, WINDS_GRAND_ODE),
];

pub mod stormzone {
    use super::*;

    pub const S: Status = Status::new_usages("Stormzone", StatusAttachMode::Character, 2, None);

    decl_status_impl_type!(Stormzone, I);
    impl StatusImpl for Stormzone {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::UpdateCost]
        }

        fn update_cost(
            &self,
            e: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            if e.has_talent_equipped() && e.is_normal_attack() && e.eff_state.can_use_once_per_round() {
                return cost
                    .try_reduce_by(1)
                    .then_some(AppliedEffectResult::ConsumeOncePerRound);
            }
            cost_type.is_switching().then_some(())?;
            cost.try_reduce_by(1).then_some(AppliedEffectResult::NoChange)
        }
    }
}

pub mod stormeye {
    use super::*;

    use crate::dispatcher_ops::RelativeCharIdx;

    pub const S: Status = Status::new_usages("Stormeye", StatusAttachMode::Summon, 2, None);

    pub struct StormeyeForceOpponentSwitch();
    impl StatusImpl for StormeyeForceOpponentSwitch {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            e.add_cmd(Command::ForceSwitchForTarget(RelativeCharIdx::ClosestTo(
                e.active_char_idx(),
            )));
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }

    compose_status_impls!(Stormeye(
        EndPhaseDealDMG(deal_elem_dmg(Element::Anemo, 2, 0)),
        StormeyeForceOpponentSwitch(),
    ));
    decl_summon_impl_type!(Stormeye, I);
}
