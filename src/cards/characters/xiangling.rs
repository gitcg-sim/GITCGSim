use super::*;

pub const C: CharCard = CharCard {
    name: "Xiangling",
    elem: Element::Pyro,
    weapon: WeaponType::Polearm,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 2,
    skills: list8![SkillId::DoughFu, SkillId::GuobaAttack, SkillId::Pyronado,],
    passive: None,
};

pub const DOUGH_FU: Skill = skill_na("Dough Fu", Element::Pyro, 2, DealDMGType::Physical);

pub const GUOBA_ATTACK: Skill = Skill {
    name: "Guoba Attack",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Pyro, 3, 0, 0),
    summon: Some(SummonSpec::One(SummonId::Guoba)),
    skill_impl: Some(&GuobaAttack()),
    ..Skill::new()
};

struct GuobaAttack();
impl SkillImpl for GuobaAttack {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        _: &StatusCollection,
        ctx: &CommandContext,
        cmds: &mut crate::data_structures::CommandList<(CommandContext, Command)>,
    ) {
        if !src_player.active_character_has_talent_equipped() {
            return;
        }
        cmds.push((
            *ctx,
            Command::DealDMG(DealDMG::new(DealDMGType::Elemental(Element::Pyro), 1, 0)),
        ));
    }
}

pub const PYRONADO: Skill = Skill {
    name: "Pyronado",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Pyro, 4, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 2, 0)),
    apply: Some(StatusId::Pyronado),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::DoughFu, DOUGH_FU),
    (SkillId::GuobaAttack, GUOBA_ATTACK),
    (SkillId::Pyronado, PYRONADO),
];

pub mod pyronado {
    use super::*;

    pub const S: Status = Status::new_usages("Pyronado", StatusAttachMode::Team, 2, None);

    decl_status_impl_type!(Pyronado, I);
    impl StatusImpl for Pyronado {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            // TODO don't know about recasting Pyronado while another Pyronado is on field
            if let SkillId::Pyronado = e.get_event_skill_ensuring_own_player()?.skill_id {
                return None;
            }
            e.cmd_deal_dmg(DealDMGType::PYRO, 2, 0);
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod guoba {
    use super::*;

    pub const S: Status = Status::new_usages("Guoba", StatusAttachMode::Summon, 2, None);

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(DealDMG::new(DealDMGType::PYRO, 2, 0));
}
