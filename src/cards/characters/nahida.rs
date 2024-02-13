use super::*;

pub const C: CharCard = CharCard {
    name: "Nahida",
    elem: Element::Dendro,
    weapon: WeaponType::Catalyst,
    faction: Faction::Sumeru,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::Akara,
        SkillId::AllSchemesToKnow,
        SkillId::AllSchemesToKnowTathata,
        SkillId::IllusoryHeart
    ],
    passive: None,
};

pub const AKARA: Skill = skill_na("Akara", Element::Dendro, 1, DealDMGType::DENDRO);

pub const ALL_SCHEMES_TO_KNOW: Skill = Skill {
    name: "All Schemes to Know",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Dendro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 2, 0)),
    commands: list8![Command::ApplyCharacterStatusToTarget(StatusId::SeedOfSkandha)],
    ..Skill::new()
};

pub const ALL_SCHEMES_TO_KNOW_TATHATA: Skill = Skill {
    name: "All Schemes to Know: Tathata",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Dendro, 5, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 3, 0)),
    commands: list8![Command::ApplyCharacterStatusToAllOpponentCharacters(
        StatusId::SeedOfSkandha
    )],
    ..Skill::new()
};

pub const ILLUSORY_HEART: Skill = Skill {
    name: "Illusory Heart",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Dendro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 4, 0)),
    apply: Some(StatusId::ShrineOfMaya),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 4] = [
    (SkillId::Akara, AKARA),
    (SkillId::AllSchemesToKnow, ALL_SCHEMES_TO_KNOW),
    (SkillId::AllSchemesToKnowTathata, ALL_SCHEMES_TO_KNOW_TATHATA),
    (SkillId::IllusoryHeart, ILLUSORY_HEART),
];

pub mod seed_of_skandha {
    use super::*;

    pub const S: Status =
        Status::new_usages("Seed of Skandha", StatusAttachMode::Character, 2, None).applies_to_opposing();

    decl_status_impl_type!(SeedOfSkandha, I);
    impl StatusImpl for SeedOfSkandha {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::DMG_REACTION & xevent_mask::DMG_INCOMING
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            if !e.is_received_dmg_into_attached_character() {
                return None;
            }
            e.add_cmd(Command::TakeDMGForAffectedBy(
                StatusId::SeedOfSkandha,
                DealDMG::new_piercing(1),
            ));
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod shrine_of_maya {
    use super::*;

    pub const S: Status = Status::new_duration("Shrine of Maya", StatusAttachMode::Team, 2);

    decl_status_impl_type!(ShrineOfMaya, I);
    impl StatusImpl for ShrineOfMaya {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingReactionDMG]
        }

        fn outgoing_reaction_dmg(
            &self,
            _: &StatusImplContext<DMGInfo>,
            _: (Reaction, Option<Element>),
            dmg: &mut DealDMG,
        ) -> Option<AppliedEffectResult> {
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }
    }
}
