use super::*;

pub const C: CharCard = CharCard {
    name: "Chongyun",
    elem: Element::Cryo,
    weapon: WeaponType::Claymore,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::Demonbane,
        SkillId::ChonghuasLayeredFrost,
        SkillId::CloudPartingStar,
    ],
    passive: None,
};

pub const DEMONBANE: Skill = skill_na("Demonbane", Element::Cryo, 2, DealDMGType::Physical);

pub const CHONGHUAS_LAYERED_FROST: Skill = Skill {
    name: "Chonghua's Layered Frost",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Cryo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 3, 0)),
    apply: Some(StatusId::ChonghuaFrostField),
    ..Skill::new()
};

pub const CLOUD_PARTING_STAR: Skill = Skill {
    name: "Cloud-Parting Star",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Cryo, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 7, 0)),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::Demonbane, DEMONBANE),
    (SkillId::ChonghuasLayeredFrost, CHONGHUAS_LAYERED_FROST),
    (SkillId::CloudPartingStar, CLOUD_PARTING_STAR),
];

pub mod chonghua_frost_field {
    use super::*;

    pub const S: Status = Status::new_duration("Chonghua Frost Field", StatusAttachMode::Team, 2)
        .talent_usages_increase(CharId::Chongyun, 1);

    decl_status_impl_type!(ChonghuaFrostField, I);
    impl StatusImpl for ChonghuaFrostField {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillType::NormalAttack) = e.skill_type() else {
                return None;
            };

            let Some(WeaponType::Claymore | WeaponType::Polearm | WeaponType::Sword) =
                e.src_char_card().map(|c| c.weapon)
            else {
                return None;
            };

            if e.has_talent_equipped() {
                dmg.dmg += 1;
            }

            dmg.infuse(DealDMGType::CRYO).then_some(AppliedEffectResult::NoChange)
        }
    }
}
