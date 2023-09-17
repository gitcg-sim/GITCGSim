use super::*;

pub const C: CharCard = CharCard {
    name: "Noelle",
    elem: Element::Geo,
    weapon: WeaponType::Claymore,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::FavoniusBladeworkMaid,
        SkillId::Breastplate,
        SkillId::SweepingTime,
    ],
    passive: None,
};

pub const FAVONIUS_BLADEWORK_MAID: Skill =
    skill_na("Favonius Bladework - Maid", Element::Geo, 2, DealDMGType::Physical);

pub const BREASTPLATE: Skill = Skill {
    name: "Breastplate",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Geo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 1, 0)),
    apply: Some(StatusId::FullPlate),
    ..Skill::new()
};

pub const SWEEPING_TIME: Skill = Skill {
    name: "Sweeping Time",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Geo, 4, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 4, 0)),
    apply: Some(StatusId::SweepingTime),
    ..Skill::new()
};

pub mod full_plate {
    use super::*;

    pub const S: Status = Status::new_shield_points("Full Plate", StatusAttachMode::Team, 2, None);

    decl_status_impl_type!(FullPlate, I);
    impl StatusImpl for FullPlate {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG | RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_NA
        }

        fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            #[inline]
            fn half_round_up(x: u8) -> u8 {
                (x / 2) + (x % 2)
            }

            if dmg.dmg_type == DealDMGType::Physical {
                dmg.dmg = half_round_up(dmg.dmg);
            }
            None
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let SkillId::FavoniusBladeworkMaid = e.get_event_skill_ensuring_own_player()?.skill_id else {
                return None;
            };
            if !e.c.eff_state.can_use_once_per_round() {
                return None;
            }

            e.add_cmd(Command::HealAll(1));
            Some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }
}

pub mod sweeping_time {
    use super::*;

    pub const S: Status = Status::new_duration("Sweeping Time", StatusAttachMode::Character, 2);

    decl_status_impl_type!(SweepingTime, I);
    impl StatusImpl for SweepingTime {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::UpdateCost | RespondsTo::OutgoingDMG]
        }

        fn update_cost(
            &self,
            e: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            let CostType::Skill(SkillId::FavoniusBladeworkMaid) = cost_type else {
                return None;
            };
            if !e.eff_state.can_use_once_per_round() {
                return None;
            }
            if cost.try_reduce_elemental_cost(1, Element::Geo) {
                Some(AppliedEffectResult::ConsumeOncePerRound)
            } else {
                None
            }
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillId::FavoniusBladeworkMaid) = e.ctx.src.skill_id() else {
                return None;
            };
            dmg.infuse(DealDMGType::GEO);
            dmg.dmg += 2;
            Some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }
}
