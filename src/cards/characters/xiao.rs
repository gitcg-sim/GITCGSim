use super::*;

pub const C: CharCard = CharCard {
    name: "Xiao",
    elem: Element::Anemo,
    weapon: WeaponType::Polearm,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::WhirlwindThrust,
        SkillId::LemniscaticWindCycling,
        SkillId::BaneOfAllEvil,
    ],
    passive: None,
};

pub const WHIRLWIND_THRUST: Skill = skill_na("Whirlwind Thrust", Element::Anemo, 2, DealDMGType::Physical);

pub const LEMNISCATIC_WIND_CYCLING: Skill = Skill {
    name: "Lemniscatic Wind Cycling",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Anemo, 3, 0, 0),
    deal_dmg: Some(DealDMG::new(DealDMGType::ANEMO, 3, 0)),
    ..Skill::new()
};

pub const BANE_OF_ALL_EVIL: Skill = Skill {
    name: "Bane of All Evil",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Anemo, 3, 0, 2),
    deal_dmg: Some(DealDMG::new(DealDMGType::ANEMO, 4, 0)),
    apply: Some(StatusId::YakshasMask),
    ..Skill::new()
};

pub mod yakshas_mask {
    use super::*;

    pub const S: Status = Status::new_usages("Yaksha's Mask", StatusAttachMode::Character, 2, None);

    decl_status_impl_type!(YakshasMask, I);
    impl StatusImpl for YakshasMask {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::UpdateCost]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !dmg.infuse(DealDMGType::ANEMO) {
                return None;
            }
            dmg.dmg += if e.is_plunging_attack() { 3 } else { 1 };
            Some(AppliedEffectResult::NoChange)
        }

        fn update_cost(
            &self,
            e: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            if !e.eff_state.can_use_once_per_round() {
                return None;
            }

            let CostType::Switching = cost_type else { return None };
            if e.status_key.char_idx() != Some(e.src_player_state.active_char_idx) {
                return None;
            }
            cost.try_reduce_unaligned_cost(1)
                .then_some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }
}
