use super::*;

pub const C: CharCard = CharCard {
    name: "Hu Tao",
    elem: Element::Pyro,
    weapon: WeaponType::Polearm,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::SecretSpearOfWangsheng,
        SkillId::GuideToAfterlife,
        SkillId::SpiritSoother
    ],
    passive: None,
};

pub const SECRET_SPEAR_OF_WANGSHENG: Skill =
    skill_na("Secret Spear of Wangsheng", Element::Pyro, 2, DealDMGType::Physical);

pub const GUIDE_TO_AFTERLIFE: Skill = Skill {
    name: "Guide to Afterlife",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Pyro, 2, 0, 0),
    apply: Some(StatusId::ParamitaPapilio),
    ..Skill::new()
};

pub const SPIRIT_SOOTHER: Skill = Skill {
    name: "Spirit Soother",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Pyro, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 4, 0)),
    ..Skill::new()
};

pub mod paramita_papilio {
    use super::*;

    pub const S: Status = Status::new_duration("Paramita Papilio", StatusAttachMode::Character, 2);

    decl_status_impl_type!(ParamitaPapilio, I);
    impl StatusImpl for ParamitaPapilio {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_NA
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillType::NormalAttack) = e.skill_type() else { return None };
            if !dmg.infuse(DealDMGType::PYRO) {
                return None;
            }
            dmg.dmg += 1;
            if e.has_talent_equipped() && Some(true) == e.get_src_character_state().map(|c| c.get_hp() <= 6) {
                dmg.dmg += 1;
            }
            Some(AppliedEffectResult::NoChange)
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let SkillType::NormalAttack = e.get_event_skill_ensuring_attached_character()?.skill_type() else {
                return None
            };
            if !e.c.is_charged_attack() {
                return None;
            };
            e.add_cmd(Command::ApplyStatusToTarget(StatusId::BloodBlossom));
            Some(AppliedEffectResult::NoChange)
        }
    }
}

pub mod blood_blossom {
    use super::*;

    pub const S: Status =
        Status::new_usages("Blood Blossom", StatusAttachMode::Character, 1, None).with_applies_to_opposing();

    pub const I: EndPhaseCommands = EndPhaseCommands(list8![Command::TakeDMG(deal_elem_dmg(Element::Pyro, 1, 0))]);
}
