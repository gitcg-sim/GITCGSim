use super::*;

pub const C: CharCard = CharCard {
    name: "Yoimiya",
    elem: Element::Pyro,
    weapon: WeaponType::Bow,
    faction: Faction::Inazuma,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::FireworkFlareUp,
        SkillId::NiwabiFireDance,
        SkillId::RyuukinSaxifrage,
    ],
    passive: None,
};

pub const FIREWORK_FLARE_UP: Skill = skill_na("Firework Flare-Up", Element::Pyro, 2, DealDMGType::Physical);

pub const NIWABI_FIRE_DANCE: Skill = Skill {
    name: "Niwabi Fire-Dance",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Pyro, 1, 0, 0),
    no_energy: true,
    apply: Some(StatusId::NiwabiEnshou),
    ..Skill::new()
};

pub const RYUUKIN_SAXIFRAGE: Skill = Skill {
    name: "Ryuukin Saxifrage",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Pyro, 4, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 4, 0)),
    apply: Some(StatusId::AurousBlaze),
    ..Skill::new()
};

pub mod niwabi_enshou {
    use super::*;
    pub const S: Status = Status::new_usages("Niwabi Enshou", StatusAttachMode::Character, 2, None);

    decl_status_impl_type!(NiwabiEnshou, I);
    impl StatusImpl for NiwabiEnshou {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillId::FireworkFlareUp) = e.skill_id() else { return None };
            dmg.infuse(DealDMGType::PYRO);
            dmg.dmg += if e.has_talent_equipped() { 2 } else { 1 };
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod aurous_blaze {
    use super::*;
    pub const S: Status = Status::new_duration("Aurous Blaze", StatusAttachMode::Team, 2);

    decl_event_handler_trait_impl!(OwnCharacterSkillEvent(AurousBlaze), I);
    impl OwnCharacterSkillEvent for AurousBlaze {
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            if e.c.is_casted_by_character(CharId::Yoimiya) {
                return None;
            }
            e.cmd_deal_dmg(DealDMGType::PYRO, 1, 0);
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
