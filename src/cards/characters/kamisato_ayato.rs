use super::*;

pub const C: CharCard = CharCard {
    name: "Kamisato Ayato",
    elem: Element::Hydro,
    weapon: WeaponType::Sword,
    faction: Faction::Inazuma,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::KamisatoArtMarobashi,
        SkillId::KamisatoArtKyouka,
        SkillId::KamisatoArtSuiyuu,
    ],
    passive: None,
};

pub const KAMISATO_ART_MAROBASHI: Skill = skill_na("Kamisato Art: Marobashi", Element::Hydro, 2, DealDMGType::Physical);

pub const KAMISATO_ART_KYOUKA: Skill = Skill {
    name: "Kamisato Art: Kyouka",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 3, 0)),
    apply: Some(StatusId::TakimeguriKanka),
    ..Skill::new()
};

pub const KAMISATO_ART_SUIYUU: Skill = Skill {
    name: "Kamisato Art: Suiyuu",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 3, 0)),
    summon: Some(SummonSpec::One(SummonId::GardenOfPurity)),
    ..Skill::new()
};

pub mod takimeguri_kanka {
    use super::*;

    pub const S: Status = Status::new_usages("Takimeguri Kanka", StatusAttachMode::Character, 2, None);

    decl_status_impl_type!(TakimeguriKanka, I);
    impl StatusImpl for TakimeguriKanka {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillType::NormalAttack) = e.skill_type() else {
                return None;
            };
            if e.has_talent_equipped() && e.dmg.target_hp <= 6 {
                dmg.dmg += 2;
            } else {
                dmg.dmg += 1;
            }
            dmg.infuse(DealDMGType::HYDRO);
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod garden_of_purity {
    use super::*;

    pub const S: Status = Status::new_usages("Garden of Purity", StatusAttachMode::Summon, 1, None);

    pub const I: GardenOfPurity = GardenOfPurity();

    pub struct GardenOfPurity();
    impl StatusImpl for GardenOfPurity {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent | RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillType::NormalAttack) = e.skill_type() else {
                return None;
            };
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }

        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
            let EventId::EndPhase = e.event_id else { return None };
            e.cmd_deal_dmg(DealDMGType::HYDRO, 2, 0);
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
