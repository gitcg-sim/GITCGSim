use super::*;

pub const C: CharCard = CharCard {
    name: "Collei",
    elem: Element::Dendro,
    weapon: WeaponType::Bow,
    faction: Faction::Sumeru,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::SupplicantsBowmanship,
        SkillId::FloralBrush,
        SkillId::TrumpCardKitty,
    ],
    passive: None,
};

pub const SUPPLICANTS_BOWMANSHIP: Skill =
    skill_na("Supplicant's Bowmanship", Element::Dendro, 2, DealDMGType::Physical);

pub const FLORAL_BRUSH: Skill = Skill {
    name: "Floral Brush",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Dendro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 3, 0)),
    skill_impl: Some(&FloralBrush()),
    ..Skill::new()
};

pub const TRUMP_CARD_KITTY: Skill = Skill {
    name: "Trump-Card Kitty",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Dendro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::CuileinAnbar)),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::SupplicantsBowmanship, SUPPLICANTS_BOWMANSHIP),
    (SkillId::FloralBrush, FLORAL_BRUSH),
    (SkillId::TrumpCardKitty, TRUMP_CARD_KITTY),
];

pub struct FloralBrush();
impl SkillImpl for FloralBrush {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        _: &StatusCollection,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        if !src_player.active_character_has_talent_equipped() {
            return;
        }
        // TODO need to check once per round
        cmds.insert(0, (*ctx, Command::ApplyStatusToTeam(StatusId::Sprout)))
    }
}

pub mod cuilein_anbar {
    use super::*;

    pub const S: Status = Status::new_summon_usages("Cuilein-Anbar", 2);

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(DealDMG::new(DealDMGType::DENDRO, 2, 0));
}

pub mod sprout {
    use super::*;
    use crate::reaction::DENDRO_REACTIONS;

    pub const S: Status = Status::new_duration("Sprout", StatusAttachMode::Team, 1);

    decl_event_handler_trait_impl!(OwnCharacterOutgoingDMGEvent(Sprout), I);
    impl OwnCharacterOutgoingDMGEvent for Sprout {
        const REACTION: bool = true;

        fn invoke(e: &mut TriggerEventContext<XEvent>, dmg: XEventDMG) -> Option<AppliedEffectResult> {
            let (reaction, _) = dmg.reaction?;
            if !DENDRO_REACTIONS.contains(reaction) {
                return None;
            }

            e.cmd_deal_dmg(DealDMGType::DENDRO, 1, 0);
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}
