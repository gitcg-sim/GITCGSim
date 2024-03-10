use super::*;

pub const C: CharCard = CharCard {
    name: "Stonehide Lawachurl",
    elem: Element::Geo,
    weapon: WeaponType::Other,
    faction: Faction::Hilichurl,
    max_health: 8,
    max_energy: 2,
    skills: list8![SkillId::PlamaLawa, SkillId::MovoLawa, SkillId::UpaShato,],
    passive: Some(Passive {
        name: "Infused Stonehide",
        apply_statuses: list8![StatusId::Stonehide, StatusId::StoneForce],
    }),
};

pub const PLAMA_LAWA: Skill = skill_na("Plama Lawa", Element::Geo, 2, DealDMGType::Physical);

pub const MOVO_LAWA: Skill = Skill {
    name: "Movo Lawa",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Geo, 3, 0, 0),
    deal_dmg: Some(DealDMG::new(DealDMGType::Physical, 5, 0)),
    ..Skill::new()
};

pub const UPA_SHATO: Skill = Skill {
    name: "Upa Shato",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Geo, 3, 0, 2),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::PlamaLawa, PLAMA_LAWA),
    (SkillId::MovoLawa, MOVO_LAWA),
    (SkillId::UpaShato, UPA_SHATO),
];

pub mod stonehide {
    use super::*;

    pub const S: Status = Status::new_usages("Stonehide", StatusAttachMode::Character, 3, None).manual_discard(true);

    decl_status_impl_type!(Stonehide, I);
    impl StatusImpl for Stonehide {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG | RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::DMG_INCOMING
        }

        fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.reduce(1);
            if dmg.dmg_type == DealDMGType::GEO {
                Some(AppliedEffectResult::ConsumeUsages(2))
            } else {
                Some(AppliedEffectResult::ConsumeUsage)
            }
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let _ = e.incoming_dmg_ensuring_attached_character()?;
            if e.c.eff_state.usages() > 0 {
                return None;
            }

            let char_idx = e.ctx_for_dmg.src.char_idx().unwrap_or(e.active_char_idx());
            e.add_cmd(Command::DeleteStatus(StatusKey::Character(
                char_idx,
                StatusId::StoneForce,
            )));
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}

pub mod stone_force {
    use super::*;

    pub const S: Status = Status::new_indef("Stone Force", StatusAttachMode::Character);

    decl_status_impl_type!(StoneForce, I);
    impl StatusImpl for StoneForce {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if e.eff_state.can_use_once_per_round() {
                dmg.dmg += 1;
                dmg.infuse(DealDMGType::GEO);
                return Some(AppliedEffectResult::ConsumeOncePerRound);
            }

            dmg.infuse(DealDMGType::GEO);
            Some(AppliedEffectResult::NoChange)
        }
    }
}
