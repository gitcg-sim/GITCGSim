use super::*;

pub const C: CharCard = CharCard {
    name: "Jean",
    elem: Element::Anemo,
    weapon: WeaponType::Sword,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![SkillId::FavoniusBladework, SkillId::GaleBlade, SkillId::DandelionBreeze,],
    passive: None,
};

pub const FAVONIUS_BLADEWORK: Skill = skill_na("Favonius Bladework", Element::Anemo, 2, DealDMGType::Physical);

pub const GALE_BLADE: Skill = Skill {
    name: "Gale Blade",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Anemo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Anemo, 3, 0)),
    commands: list8![Command::SwitchNextForTarget],
    ..Skill::new()
};

pub const DANDELION_BREEZE: Skill = Skill {
    name: "Dandelion Breeze",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Anemo, 4, 0, 3),
    summon: Some(SummonSpec::One(SummonId::DandelionField)),
    commands: list8![Command::HealAll(2)],
    ..Skill::new()
};

pub mod dandelion_field {
    use super::*;

    pub const S: Status = Status::new_usages("Dandelion Field", StatusAttachMode::Summon, 2, None);

    pub const I: DandelionField = DandelionField();
    pub struct DandelionField();
    impl StatusImpl for DandelionField {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent | RespondsTo::OutgoingDMG]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(Element::Anemo) = dmg.dmg_type.element() else { return None };
            if !e
                .src_player_state
                .char_states
                .iter()
                .any(|&c| c.char_id == CharId::Jean && c.has_talent_equipped())
            {
                return None;
            }
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            e.cmd_deal_dmg(DealDMGType::Elemental(Element::Anemo), 2, 0);
            e.out_cmds.push((*e.ctx_for_dmg, Command::Heal(1)));
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
