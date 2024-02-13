use super::*;

pub const C: CharCard = CharCard {
    name: "Ningguang",
    elem: Element::Geo,
    weapon: WeaponType::Catalyst,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 3,
    skills: list8![SkillId::SparklingScatter, SkillId::JadeScreen, SkillId::Starshatter,],
    passive: None,
};

pub const SPARKLING_SCATTER: Skill = skill_na("Sparkling Scatter", Element::Geo, 1, DealDMGType::GEO);

pub const JADE_SCREEN: Skill = Skill {
    name: "Jade Screen",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Dendro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 2, 0)),
    apply: Some(StatusId::JadeScreen),
    ..Skill::new()
};

pub const STARSHATTER: Skill = Skill {
    name: "Starshatter",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Dendro, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 6, 0)),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::SparklingScatter, SPARKLING_SCATTER),
    (SkillId::JadeScreen, JADE_SCREEN),
    (SkillId::Starshatter, STARSHATTER),
];

pub mod jade_screen {
    use super::*;

    pub const S: Status =
        Status::new_usages("Jade Screen", StatusAttachMode::Team, 2, None).casted_by_character(CharId::Ningguang);

    decl_status_impl_type!(JadeScreen, I);
    impl StatusImpl for JadeScreen {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG | RespondsTo::OutgoingDMG]
        }

        fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if dmg.dmg >= 2 {
                dmg.dmg -= 1;
                return Some(AppliedEffectResult::ConsumeUsage);
            }
            None
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if let Some(SkillId::Starshatter) = e.skill_id() {
                if e.has_talent_equipped() {
                    dmg.dmg += 3;
                } else {
                    dmg.dmg += 2;
                }
                Some(AppliedEffectResult::NoChange)
            } else {
                if !e.has_talent_equipped() {
                    return None;
                }
                let Some(Element::Geo) = dmg.dmg_type.element() else {
                    return None;
                };
                dmg.dmg += 2;
                Some(AppliedEffectResult::NoChange)
            }
        }
    }
}
