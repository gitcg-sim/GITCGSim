use crate::std_subset::cmp::Ordering;

use super::*;

pub const C: CharCard = CharCard {
    name: "Diluc",
    elem: Element::Pyro,
    weapon: WeaponType::Claymore,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 3,
    skills: list8![SkillId::TemperedSword, SkillId::SearingOnslaught, SkillId::Dawn],
    passive: Some(Passive::new_hidden().status(StatusId::SearingOnslaughtCounter)),
};

pub const TEMPERED_SWORD: Skill = skill_na("Tempered Sword", Element::Pyro, 2, DealDMGType::Physical);

pub const SEARING_ONSLAUGHT: Skill = Skill {
    name: "Searing Onslaught",
    skill_type: SkillType::ElementalSkill,
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 3, 0)),
    cost: cost_elem(Element::Pyro, 3, 0, 0),
    ..Skill::new()
};

pub const DAWN: Skill = Skill {
    name: "Dawn",
    skill_type: SkillType::ElementalBurst,
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 8, 0)),
    cost: cost_elem(Element::Pyro, 4, 0, 3),
    apply: Some(StatusId::PyroInfusion),
    ..Skill::new()
};

pub mod searing_onslaught_counter {
    use super::*;

    pub const S: Status = Status::new_indef("[Searing Onslaught Counter]", StatusAttachMode::Character)
        .counter(CounterSpec::new("[Searing Onslaught Counter]", 0).resets_at_turn_end(true));

    decl_status_impl_type!(SearingOnslaughtCounter, I);
    impl StatusImpl for SearingOnslaughtCounter {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::UpdateCost]
        }

        fn update_cost(
            &self,
            e: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            if !e.has_talent_equipped() {
                return None;
            }
            let CostType::Skill(SkillId::SearingOnslaught) = cost_type else {
                return None;
            };
            let 1 = e.eff_state.get_counter() else { return None };
            cost.try_reduce_elemental_cost(1, Element::Pyro)
                .then_some(AppliedEffectResult::NoChange)
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillId::SearingOnslaught) = e.skill_id() else {
                return None;
            };
            let counter = e.eff_state.get_counter();
            match counter.cmp(&2) {
                Ordering::Less => Some(AppliedEffectResult::SetCounter(counter + 1)),
                Ordering::Equal => {
                    dmg.dmg += 2;
                    Some(AppliedEffectResult::SetCounter(3))
                }
                Ordering::Greater => None,
            }
        }
    }
}

pub mod pyro_infusion {
    use super::*;

    pub const S: Status = Status::new_duration("Pyro Infusion", StatusAttachMode::Character, 2);

    decl_status_impl_type!(PyroInfusion, I);
    impl StatusImpl for PyroInfusion {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillId::TemperedSword) = e.skill_id() else {
                return None;
            };
            dmg.infuse(DealDMGType::Elemental(Element::Pyro))
                .then_some(AppliedEffectResult::NoChange)
        }
    }
}
