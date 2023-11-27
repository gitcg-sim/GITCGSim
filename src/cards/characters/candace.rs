use super::*;

pub const C: CharCard = CharCard {
    name: "Candace",
    elem: Element::Hydro,
    weapon: WeaponType::Polearm,
    faction: Faction::Sumeru,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::GleamingSpearGuardianStance,
        SkillId::SacredRiteHeronsSanctum,
        SkillId::SacredRiteWagtailsTide,
    ],
    passive: None,
};

pub const GLEAMING_SPEAR_GUARDIAN_STANCE: Skill = skill_na(
    "Gleaming Spear - Guardian Stance",
    Element::Hydro,
    2,
    DealDMGType::Physical,
);

pub const SACRED_RITE_HERONS_SANCTUM: Skill = Skill {
    name: "Sacred Rite: Heron's Sanctum",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    apply: Some(StatusId::HeronShield),
    ..Skill::new()
};

pub const SACRED_RITE_WAGTAILS_TIDE: Skill = Skill {
    name: "Sacred Rite: Wagtail's Tide",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 2, 0)),
    apply: Some(StatusId::PrayerOfTheCrimsonCrown),
    ..Skill::new()
};

pub const HERON_STRIKE: Skill = Skill {
    name: "Heron Strike",
    skill_type: SkillType::ElementalSkill,
    cost: Cost::ZERO,
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 3, 0)),
    ..Skill::new()
};

pub mod heron_shield {
    use super::*;

    pub const S: Status =
        Status::new_shield_points("Heron Shield", StatusAttachMode::Character, 2, None).prepare_skill(1);

    pub const I: PreparedSkill = PreparedSkill::new(SkillId::HeronStrike);
}

pub mod prayer_of_the_crimson_crown {
    use super::*;

    pub const S: Status = Status::new_duration("Prayer of the Crimson Crown", StatusAttachMode::Team, 2)
        .casted_by_character(CharId::Candace);

    decl_status_impl_type!(PrayerOfTheCrimsonCrown, I);
    pub struct PrayerOfTheCrimsonCrownOutgoingDMG();
    impl StatusImpl for PrayerOfTheCrimsonCrownOutgoingDMG {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let mut found = false;
            if let Some(SkillType::NormalAttack) = e.skill_type() {
                dmg.dmg += 1;
                found = true;
            }

            if let Some(WeaponType::Sword | WeaponType::Claymore | WeaponType::Polearm) = e.weapon_type() {
                found |= dmg.infuse(DealDMGType::HYDRO);
            }
            found.then_some(AppliedEffectResult::NoChange)
        }
    }

    pub struct PrayerOfTheCrimsonCrownNormalAttack();
    impl OwnCharacterSkillEvent for PrayerOfTheCrimsonCrownNormalAttack {
        const SKILL_TYPES: EnumSet<SkillType> = enum_set![SkillType::NormalAttack];
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            if !e.c.has_talent_equipped() || !e.c.eff_state.can_use_once_per_round() {
                return None;
            }

            e.cmd_deal_dmg(DealDMGType::HYDRO, 1, 0);
            Some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }

    compose_status_impls!(PrayerOfTheCrimsonCrown(
        PrayerOfTheCrimsonCrownOutgoingDMG(),
        OwnCharacterSkillEventI(PrayerOfTheCrimsonCrownNormalAttack()),
    ));
}
