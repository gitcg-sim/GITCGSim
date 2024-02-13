use super::*;

pub const C: CharCard = CharCard {
    name: "Xingqiu",
    elem: Element::Hydro,
    weapon: WeaponType::Sword,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 2,
    skills: list8![SkillId::GuhuaStyle, SkillId::FatalRainscreen, SkillId::Raincutter,],
    passive: None,
};

pub const GUHUA_STYLE: Skill = skill_na("Guhua Style", Element::Hydro, 2, DealDMGType::Physical);

pub const FATAL_RAINSCREEN: Skill = Skill {
    name: "Fatal Rainscreen",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 2, 0)),
    apply: Some(StatusId::RainSword),
    commands: list8![Command::ApplyElementToSelf(Element::Hydro),],
    ..Skill::new()
};

pub const RAINCUTTER: Skill = Skill {
    name: "Raincutter",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 1, 0)),
    apply: Some(StatusId::RainbowBladework),
    commands: list8![Command::ApplyElementToSelf(Element::Hydro),],
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::GuhuaStyle, GUHUA_STYLE),
    (SkillId::FatalRainscreen, FATAL_RAINSCREEN),
    (SkillId::Raincutter, RAINCUTTER),
];

pub mod rain_sword {
    use super::*;

    pub const S: Status =
        Status::new_usages("Rain Sword", StatusAttachMode::Team, 2, None).talent_usages_increase(CharId::Xingqiu, 1);

    decl_status_impl_type!(RainSword, I);
    impl StatusImpl for RainSword {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG]
        }

        fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if dmg.dmg >= 3 {
                dmg.dmg -= 1;
                return Some(AppliedEffectResult::ConsumeUsage);
            }
            None
        }
    }
}

pub mod rainbow_bladework {
    use super::*;

    pub const S: Status = Status::new_usages("Rainbow Bladework", StatusAttachMode::Team, 3, None);

    decl_event_handler_trait_impl!(OwnCharacterSkillEvent(RainbowBladework), I);
    impl OwnCharacterSkillEvent for RainbowBladework {
        const SKILL_TYPES: EnumSet<SkillType> = enum_set![SkillType::NormalAttack];
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            e.cmd_deal_dmg(DealDMGType::HYDRO, 2, 0);
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
