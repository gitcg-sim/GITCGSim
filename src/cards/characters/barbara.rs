use super::*;

pub const C: CharCard = CharCard {
    name: "Barbara",
    elem: Element::Hydro,
    weapon: WeaponType::Catalyst,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::WhisperOfWater,
        SkillId::LetTheShowBegin,
        SkillId::ShiningMiracle,
    ],
    passive: None,
};

pub const WHISPER_OF_WATER: Skill = skill_na("Whisper of Water", Element::Hydro, 1, DealDMGType::HYDRO);

pub const LET_THE_SHOW_BEGIN: Skill = Skill {
    name: "Let the Show Begin♪",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::MelodyLoop)),
    ..Skill::new()
};

pub const SHINING_MIRACLE: Skill = Skill {
    name: "Shining Miracle♪",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 3),
    commands: list8![Command::HealAll(4),],
    ..Skill::new()
};

pub mod melody_loop {
    use super::*;

    pub const S: Status =
        Status::new_usages("Melody Loop", StatusAttachMode::Summon, 2, None).with_casted_by_character(CharId::Barbara);

    pub const I: MelodyLoop = MelodyLoop();

    pub struct MelodyLoop();

    impl StatusImpl for MelodyLoop {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::UpdateCost | RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn update_cost(
            &self,
            e: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            if !e.has_talent_equipped() {
                return None;
            }

            let CostType::Switching = cost_type else { return None };

            if !e.eff_state.can_use_once_per_round() {
                return None;
            }

            cost.try_reduce_by(1)
                .then_some(AppliedEffectResult::ConsumeOncePerRound)
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            e.add_cmd(Command::HealAll(1));
            e.add_cmd(Command::ApplyElementToSelf(Element::Hydro));
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
