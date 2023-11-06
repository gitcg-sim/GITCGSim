use super::*;

pub const C: CharCard = CharCard {
    name: "Sucrose",
    elem: Element::Anemo,
    weapon: WeaponType::Catalyst,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::WindSpiritCreation,
        SkillId::AstableAnemohypostasisCreation6308,
        SkillId::ForbiddenCreationIsomer75TypeII,
    ],
    passive: None,
};

pub const WIND_SPIRIT_CREATION: Skill = skill_na("Wind Spirit Creation", Element::Anemo, 1, DealDMGType::ANEMO);

pub const ASTABLE_ANEMOHYPOSTASIS_CREATION_6308: Skill = Skill {
    name: "Astable Anemohypostasis Creation - 6308",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Anemo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Anemo, 3, 0)),
    commands: list8![Command::ForceSwitchForTarget(RelativeSwitchType::Previous)],
    ..Skill::new()
};

pub const FORBIDDEN_CREATION_ISOMER_75_TYPE_II: Skill = Skill {
    name: "Forbidden Creation - Isomer 75 / Type II",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Anemo, 1, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Anemo, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::LargeWindSpirit)),
    ..Skill::new()
};

pub mod large_wind_spirit {
    use super::*;

    pub const S: Status =
        Status::new_usages("Large Wind Spirit", StatusAttachMode::Summon, 3, None).with_counter(CounterSpec {
            name: "Infusion",
            default_value: Element::Anemo as u8,
            resets_at_turn_end: false,
        });

    pub struct LargeWindSpiritEndPhase();
    trigger_event_impl!(LargeWindSpiritEndPhase, [EndPhase], |e| {
        let deal_dmg = DealDMGType::Elemental(Element::VALUES[e.c.eff_state.get_counter() as usize]);
        e.cmd_deal_dmg(deal_dmg, 2, 0);
        Some(AppliedEffectResult::ConsumeUsage)
    });

    pub struct LargeWindSpritReactionDMG();
    impl OwnCharacterOutgoingDMGEvent for LargeWindSpritReactionDMG {
        const REACTION: bool = true;

        fn invoke(e: &mut TriggerEventContext<XEvent>, dmg: XEventDMG) -> Option<AppliedEffectResult> {
            if e.c.eff_state.get_counter() != Element::Anemo as u8 {
                return None;
            }
            if let Some((Reaction::Swirl, Some(elem))) = dmg.reaction {
                Some(AppliedEffectResult::SetCounter(elem as u8))
            } else {
                Some(AppliedEffectResult::NoChange)
            }
        }
    }

    compose_status_impls!(LargeWindSpirit(
        LargeWindSpiritEndPhase(),
        OwnCharacterOutgoingDMGEventI(LargeWindSpritReactionDMG())
    ));
    decl_summon_impl_type!(LargeWindSpirit, I);
}
