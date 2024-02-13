use super::*;

pub const C: CharCard = CharCard {
    name: "Kaeya",
    elem: Element::Cryo,
    weapon: WeaponType::Sword,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 2,
    skills: list8![SkillId::CeremonialBladework, SkillId::Frostgnaw, SkillId::GlacialWaltz,],
    passive: None,
};

pub const CEREMONIAL_BLADEWORK: Skill = skill_na("Ceremonial Bladework", Element::Pyro, 2, DealDMGType::Physical);

pub const FROSTGNAW: Skill = Skill {
    name: "Frostgnaw",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Cryo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 3, 0)),
    ..Skill::new()
};

pub const GLACIAL_WALTZ: Skill = Skill {
    name: "Glacial Waltz",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Cryo, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 1, 0)),
    apply: Some(StatusId::Icicle),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::CeremonialBladework, CEREMONIAL_BLADEWORK),
    (SkillId::Frostgnaw, FROSTGNAW),
    (SkillId::GlacialWaltz, GLACIAL_WALTZ),
];

pub mod icicle {
    use super::*;
    pub const S: Status = Status::new_usages("Icicle", StatusAttachMode::Team, 3, None);

    decl_status_impl_type!(Icicle, I);
    impl StatusImpl for Icicle {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::Switched]
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            let EventId::Switched = e.event_id else { return None };
            e.cmd_deal_dmg(DealDMGType::CRYO, 2, 0);
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
