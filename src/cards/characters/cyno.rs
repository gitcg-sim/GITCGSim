use super::*;

pub const C: CharCard = CharCard {
    name: "Cyno",
    elem: Element::Electro,
    weapon: WeaponType::Polearm,
    faction: Faction::Sumeru,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::InvokersSpear,
        SkillId::SecretRiteChasmicSoulfarer,
        SkillId::SacredRiteWolfsSwiftness,
    ],
    passive: Some(Passive::new("Lawful Enforcer").status(StatusId::PactswornPathclearer)),
};

pub const INVOKERS_SPEAR: Skill = skill_na("Invoker's Spear", Element::Electro, 2, DealDMGType::Physical);

pub const SECRET_RITE_CHASMIC_SOULFARER: Skill = Skill {
    name: "Secret Rite: Chasmic Soulfarer",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Electro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 3, 0)),
    ..Skill::new()
};

pub const SACRED_RITE_WOLFS_SWIFTNESS: Skill = Skill {
    name: "Sacred Rite: Wolf's Swiftness",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Electro, 4, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 4, 0)),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::InvokersSpear, INVOKERS_SPEAR),
    (SkillId::SecretRiteChasmicSoulfarer, SECRET_RITE_CHASMIC_SOULFARER),
    (SkillId::SacredRiteWolfsSwiftness, SACRED_RITE_WOLFS_SWIFTNESS),
];

pub mod pactsworn_pathclearer {
    use super::*;

    pub const S: Status = Status::new_indef("Pactsworn Pathclearer", StatusAttachMode::Character)
        .counter(CounterSpec::new("Indwelling Level", 0));

    #[inline]
    fn increase_indwelling_level(level: u8, increase: u8) -> u8 {
        let level1 = level + increase;
        if level1 >= 6 {
            level1 - 4
        } else {
            level1
        }
    }

    decl_status_impl_type!(PactswornPathclearer, I);
    impl StatusImpl for PactswornPathclearer {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::TriggerEvent | RespondsTo::TriggerXEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_BURST
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let level = e.eff_state.get_counter();
            let mut changed = false;
            if level >= 2 && dmg.infuse(DealDMGType::Elemental(Element::Electro)) {
                changed = true;
            }
            if level >= 4 {
                dmg.dmg += 2;
                changed = true;
            }

            changed.then_some(AppliedEffectResult::NoChange)
        }

        // TODO need level decrease check for equipping Talent Card
        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            let EventId::EndPhase = e.event_id else { return None };
            let level = e.c.eff_state.get_counter();
            Some(AppliedEffectResult::SetCounter(increase_indwelling_level(level, 1)))
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let SkillId::SacredRiteWolfsSwiftness = e.get_event_skill_ensuring_attached_character()?.skill_id else {
                return None;
            };
            let level = e.c.eff_state.get_counter();
            Some(AppliedEffectResult::SetCounter(increase_indwelling_level(level, 2)))
        }
    }
}
