use super::*;

pub const C: CharCard = CharCard {
    name: "Mona",
    elem: Element::Hydro,
    weapon: WeaponType::Catalyst,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::RippleOfFate,
        SkillId::MirrorReflectionOfDoom,
        SkillId::StellarisPhantasm,
    ],
    passive: Some(Passive::new("Illusory Torrent").status(StatusId::IllusoryTorrent)),
};

pub const RIPPLE_OF_FATE: Skill = skill_na("Ripple of Fate", Element::Hydro, 1, DealDMGType::HYDRO);

pub const MIRROR_REFLECTION_OF_DOOM: Skill = Skill {
    name: "Mirror Reflection of Doom",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 1, 0)),
    summon: Some(SummonSpec::One(SummonId::Reflection)),
    ..Skill::new()
};

pub const STELLARIS_PHANTASM: Skill = Skill {
    name: "Stellaris Phantasm",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 4, 0)),
    apply: Some(StatusId::IllusoryBubble),
    ..Skill::new()
};

pub mod illusory_bubble {
    use super::*;

    pub const S: Status = Status::new_indef("Illusory Bubble", StatusAttachMode::Team);

    decl_status_impl_type!(IllusoryBubble, I);
    impl StatusImpl for IllusoryBubble {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::MultiplyOutgoingDMG]
        }

        fn multiply_dmg(&self, e: &StatusImplContext<DMGInfo>, mult: &mut u8) -> Option<AppliedEffectResult> {
            let Some(..) = e.skill_id() else {
                return None;
            };
            *mult *= 2;
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}

pub mod reflection {
    use super::*;

    pub const S: Status = Status::new_usages("Reflection", StatusAttachMode::Summon, 1, None).manual_discard(true);

    pub const I: Reflection = Reflection();
    pub struct Reflection();
    impl StatusImpl for Reflection {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent | RespondsTo::IncomingDMG]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !e.eff_state.no_usages() {
                return dmg.try_reduce(1, AppliedEffectResult::ConsumeUsage);
            }
            None
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            e.cmd_deal_dmg(DealDMGType::Elemental(Element::Hydro), 1, 0);
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}

pub mod illusory_torrent {
    use crate::reaction::HYDRO_REACTIONS;

    use super::*;
    pub const S: Status = Status::new_indef("Illusory Torrent", StatusAttachMode::Character);

    decl_status_impl_type!(IllusoryTorrent, I);
    impl StatusImpl for IllusoryTorrent {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::SwitchIsFastAction | RespondsTo::OutgoingReactionDMG]
        }

        fn outgoing_reaction_dmg(
            &self,
            e: &StatusImplContext<DMGInfo>,
            (reaction, _): (Reaction, Option<Element>),
            dmg: &mut DealDMG,
        ) -> Option<AppliedEffectResult> {
            if !HYDRO_REACTIONS.contains(reaction) {
                return None;
            }
            if !e.has_talent_equipped() {
                return None;
            }
            dmg.dmg += 2;
            Some(AppliedEffectResult::NoChange)
        }

        fn switch_is_fast_action(&self, eff_state: &AppliedEffectState, res: &mut bool) -> Option<AppliedEffectResult> {
            if !eff_state.can_use_once_per_round() {
                return None;
            }

            *res = true;
            Some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }
}
