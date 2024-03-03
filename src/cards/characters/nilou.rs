use super::*;

pub const C: CharCard = CharCard {
    name: "Nilou",
    elem: Element::Hydro,
    weapon: WeaponType::Sword,
    faction: Faction::Sumeru,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::DanceOfSamser,
        SkillId::DanceOfHaftkarsvar,
        SkillId::DanceOfAbzendegiDistantDreamsListeningSpring,
    ],
    passive: None,
};

pub const DANCE_OF_SAMSER: Skill = skill_na("Dance of Samser", Element::Hydro, 2, DealDMGType::Physical);

pub const DANCE_OF_HAFTKARSVAR: Skill = Skill {
    name: "Dance of Haftkarsvar",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 3, 0)),
    skill_impl: Some(&DanceOfHaftkarsvar()),
    ..Skill::new()
};

pub const DANCE_OF_ABZENDEGI_DISTANT_DREAMS_LISTENING_SPRING: Skill = Skill {
    name: "Dance of Abzendegi: Distant Dreams, Listening Spring",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 2, 0)),
    commands: list8![Command::ApplyCharacterStatusToTarget(StatusId::LingeringAeon),],
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::DanceOfSamser, DANCE_OF_SAMSER),
    (SkillId::DanceOfHaftkarsvar, DANCE_OF_HAFTKARSVAR),
    (
        SkillId::DanceOfAbzendegiDistantDreamsListeningSpring,
        DANCE_OF_ABZENDEGI_DISTANT_DREAMS_LISTENING_SPRING,
    ),
];

pub struct DanceOfHaftkarsvar();
impl SkillImpl for DanceOfHaftkarsvar {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        _: &StatusCollection,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        if src_player
            .char_states
            .iter_all()
            .any(|c| !matches!(c.char_id.get_char_card().elem, Element::Hydro | Element::Dendro))
        {
            return;
        }

        cmds.push((*ctx, Command::ApplyStatusToTeam(StatusId::GoldenChalicesBounty)));
    }
}

pub mod bountiful_core {
    use super::*;

    pub const S: Status =
        Status::new_usages("Bountiful Core", StatusAttachMode::Summon, 1, Some(3)).casted_by_character(CharId::Nilou);

    decl_summon_impl_type!(BountifulCore, I);
    trigger_event_impl!(BountifulCore, [DeclareEndOfRound, EndPhase], |e| {
        let dmg = if e.c.has_talent_equipped() { 3 } else { 2 };
        match e.event_id {
            EventId::DeclareEndOfRound if e.c.eff_state.get_usages() >= 2 => {
                e.cmd_deal_dmg(DealDMGType::DENDRO, dmg, 0);
                Some(AppliedEffectResult::ConsumeUsage)
            }
            EventId::EndPhase => {
                e.cmd_deal_dmg(DealDMGType::DENDRO, dmg, 0);
                Some(AppliedEffectResult::ConsumeUsage)
            }
            _ => None,
        }
    });
}

pub mod golden_chalices_bounty {
    use super::*;

    pub const S: Status = Status::new_indef("Golden Chalice's Bounty", StatusAttachMode::Team);

    decl_status_impl_type!(GoldenChalicesBounty, I);
    impl StatusImpl for GoldenChalicesBounty {
        // Implemented elsewhere
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![]
        }
    }
}

pub mod lingering_aeon {
    use super::*;

    pub const S: Status =
        Status::new_usages("Lingering Aeon", StatusAttachMode::Character, 1, None).applies_to_opposing();

    pub const I: EndPhaseTakeDMG =
        EndPhaseTakeDMG::new(TakeDMGCharacter::Attached, deal_elem_dmg(Element::Hydro, 3, 0));
}
