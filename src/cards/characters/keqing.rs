use super::*;

pub const C: CharCard = CharCard {
    name: "Keqing",
    elem: Element::Electro,
    weapon: WeaponType::Sword,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::YunlaiSwordsmanship,
        SkillId::StellarRestoration,
        SkillId::StarwardSword,
    ],
    passive: None,
};

pub const YUNLAI_SWORDSMANSHIP: Skill = skill_na("Yunlai Swordsmanship", Element::Electro, 2, DealDMGType::Physical);

pub const STELLAR_RESTORATION: Skill = Skill {
    name: "Stellar Restoration",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Electro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 3, 0)),
    commands: list8![Command::StellarRestorationFromSkill],
    ..Skill::new()
};

pub const STARWARD_SWORD: Skill = Skill {
    name: "Starward Sword",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Electro, 4, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 4, 3)),
    ..Skill::new()
};

pub mod electro_infusion {
    use super::*;
    pub const S: Status = Status::new_duration("Electro Infusion", StatusAttachMode::Character, 2)
        .talent_usages_increase(CharId::Keqing, 1);

    decl_status_impl_type!(ElectroInfusion, I);
    impl StatusImpl for ElectroInfusion {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, _: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if dmg.infuse(DealDMGType::ELECTRO) {
                return Some(AppliedEffectResult::NoChange);
            }
            None
        }
    }
}
