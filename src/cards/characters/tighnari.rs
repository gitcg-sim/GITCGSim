use super::*;

pub const C: CharCard = CharCard {
    name: "Tighnari",
    elem: Element::Dendro,
    weapon: WeaponType::Bow,
    faction: Faction::Sumeru,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::KhandaBarrierBuster,
        SkillId::VijnanaPhalaMine,
        SkillId::FashionersTanglevineShaft,
    ],
    passive: None,
};

pub const KHANDA_BARRIER_BUSTER: Skill = skill_na("Khanda Barrier-Buster", Element::Dendro, 2, DealDMGType::Physical);

pub const VIJNANA_PHALA_MINE: Skill = Skill {
    name: "Vijnana-Phala Mine",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Dendro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 2, 0)),
    apply: Some(StatusId::VijnanaSuffusion),
    ..Skill::new()
};

pub const FASHIONERS_TANGLEVINE_SHAFT: Skill = Skill {
    name: "Fashioner's Tanglevine Shaft",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Dendro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 4, 1)),
    ..Skill::new()
};

pub mod vijnana_suffusion {
    use super::*;

    pub const S: Status = Status::new_usages("Vijnana Suffusion", StatusAttachMode::Character, 2, None);

    decl_status_impl_type!(VijnanaSuffusion, I);
    impl StatusImpl for VijnanaSuffusion {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::UpdateCost | RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::DMG_OUTGOING
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !e.is_charged_attack() {
                return None;
            }

            if dmg.infuse(DealDMGType::DENDRO) {
                Some(AppliedEffectResult::ConsumeUsage)
            } else {
                None
            }
        }

        fn update_cost(
            &self,
            e: &StatusImplContext,
            cost: &mut Cost,
            cost_type: CostType,
        ) -> Option<AppliedEffectResult> {
            if !e.has_talent_equipped() || !e.is_charged_attack() {
                return None;
            }

            let CostType::Skill(SkillId::KhandaBarrierBuster) = cost_type else {
                return None;
            };

            if cost.try_reduce_unaligned_cost(1) {
                Some(AppliedEffectResult::NoChange)
            } else {
                None
            }
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            if !e.is_outgoing_dmg() {
                return None;
            }
            if !e.c.is_charged_attack() {
                return None;
            }

            e.add_cmd(Command::Summon(SummonId::ClusterbloomArrow));
            Some(AppliedEffectResult::NoChange)
        }
    }
}

pub mod clusterbloom_arrow {
    use super::*;

    pub const S: Status = Status::new_usages("Clusterbloom Arrow", StatusAttachMode::Summon, 1, Some(2));

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(DealDMG::new(DealDMGType::DENDRO, 1, 0));
}
