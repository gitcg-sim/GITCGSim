use crate::data_structures::CommandList;

use super::*;

use std::cmp::min;

pub const C: CharCard = CharCard {
    name: "Eula",
    elem: Element::Cryo,
    weapon: WeaponType::Claymore,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::FavoniusBladeworkEdel,
        SkillId::IcetideVortex,
        SkillId::GlacialIllumination,
    ],
    passive: None,
};

pub const FAVONIUS_BLADEWORK_EDEL: Skill =
    skill_na("Favonius Bladework - Edel", Element::Cryo, 2, DealDMGType::Physical);

pub const ICETIDE_VORTEX: Skill = Skill {
    name: "Icetide Vortex",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Cryo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 2, 0)),
    skill_impl: Some(&IcetideVortex()),
    ..Skill::new()
};

struct IcetideVortex();
impl SkillImpl for IcetideVortex {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        let Some(char_idx) = ctx.src.char_idx() else { return };
        if !src_player
            .status_collection
            .has_character_status(char_idx, StatusId::Grimheart)
        {
            cmds.push((*ctx, Command::ApplyStatusToActiveCharacter(StatusId::Grimheart)))
        }
    }
}

pub const GLACIAL_ILLUMINATION: Skill = Skill {
    name: "Glacial Illumination",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Cryo, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 2, 0)),
    summon: Some(SummonSpec::One(SummonId::LightfallSword)),
    ..Skill::new()
};

pub mod grimheart {
    use super::*;

    pub const S: Status = Status::new_indef("Grimheart", StatusAttachMode::Character);

    decl_status_impl_type!(Grimheart, I);
    impl StatusImpl for Grimheart {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillId::IcetideVortex) = e.skill_id() else {
                return None;
            };
            dmg.dmg += 2;
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}

pub mod lightfall_sword {
    use super::*;

    pub const S: Status =
        Status::new_indef("Lightfall Sword", StatusAttachMode::Summon).with_counter(CounterSpec::new("Zeal", 0));

    pub const I: LightfallSword = LightfallSword();
    pub struct LightfallSword();
    impl StatusImpl for LightfallSword {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::GainsEnergy | RespondsTo::TriggerEvent | RespondsTo::TriggerXEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & (xevent_mask::SKILL_NA | xevent_mask::SKILL_SKILL)
        }

        fn gains_energy(&self, _: &StatusImplContext, ctx_for_skill: &CommandContext, gains_energy: &mut bool) -> bool {
            let Some(SkillId::FavoniusBladeworkEdel | SkillId::IcetideVortex) = ctx_for_skill.src.skill_id() else {
                return false;
            };
            *gains_energy = false;
            true
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            let EventId::EndPhase = e.event_id else { return None };
            let stacks = e.c.eff_state.get_counter();
            let dmg = 2 + stacks;
            e.cmd_deal_dmg(DealDMGType::Physical, dmg, 0);
            Some(AppliedEffectResult::DeleteSelf)
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let (SkillId::FavoniusBladeworkEdel | SkillId::IcetideVortex) =
                e.get_event_skill_ensuring_own_player()?.skill_id
            else {
                return None;
            };
            let stacks = e.c.eff_state.get_counter();
            let new_stacks = min(AppliedEffectState::MAX_COUNTER, stacks + 2);
            Some(AppliedEffectResult::SetCounter(new_stacks))
        }
    }
}
