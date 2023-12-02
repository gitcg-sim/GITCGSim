use super::*;

pub const C: CharCard = CharCard {
    name: "Arataki Itto",
    elem: Element::Geo,
    weapon: WeaponType::Claymore,
    faction: Faction::Inazuma,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::FightClubLegend,
        SkillId::MasatsuZetsugiAkaushiBurst,
        SkillId::RoyalDescentBeholdIttoTheEvil,
    ],
    passive: None,
};

pub const FIGHT_CLUB_LEGEND: Skill = skill_na("Fight Club Legend", Element::Geo, 2, DealDMGType::Physical);

pub const MASATSU_ZETSUGI_AKAUSHI_BURST: Skill = Skill {
    name: "Masatsu Zetsugi: Akaushi Burst",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Geo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 3, 0)),
    summon: Some(SummonSpec::One(SummonId::Ushi)),
    ..Skill::new()
};

pub const ROYAL_DESCENT_BEHOLD_ITTO_THE_EVIL: Skill = Skill {
    name: "Royal Descent: Behold, Itto the Evil!",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Geo, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Geo, 5, 0)),
    apply: Some(StatusId::RagingOniKing),
    ..Skill::new()
};

pub mod ushi {
    use super::*;

    pub const S: Status = Status::new_usages("Ushi", StatusAttachMode::Summon, 1, None)
        .counter(CounterSpec::new("Ushi Trigger", 1))
        .manual_discard(true);

    pub const I: Ushi = Ushi();

    pub struct Ushi();
    impl StatusImpl for Ushi {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG | RespondsTo::TriggerEvent | RespondsTo::TriggerXEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::DMG_INCOMING
        }

        fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !e.eff_state.no_usages() {
                return dmg.try_reduce(1, AppliedEffectResult::ConsumeUsage);
            }
            None
        }

        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
            let EventId::EndPhase = e.event_id else { return None };
            e.cmd_deal_dmg(DealDMGType::GEO, 1, 0);
            Some(AppliedEffectResult::DeleteSelf)
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            if !e.is_received_dmg() {
                return None;
            }
            if e.c.eff_state.get_counter() == 0 {
                return None;
            };
            let Some((char_idx, _)) = e.find_chararacter_for(|c| c.char_id == CharId::AratakiItto) else {
                return None;
            };
            e.add_cmd(Command::ApplyStatusToCharacter(
                StatusId::SuperlativeSuperstrength,
                char_idx.into(),
            ));
            Some(AppliedEffectResult::SetCounter(0))
        }
    }
}

pub mod superlative_superstrength {
    use super::*;

    pub const S: Status = Status::new_usages("Superlative Superstrength", StatusAttachMode::Character, 1, None);

    decl_status_impl_type!(SuperlativeSuperstrength, I);
    impl StatusImpl for SuperlativeSuperstrength {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::UpdateCost]
        }

        fn update_cost(&self, e: &StatusImplContext, cost: &mut Cost, _: CostType) -> Option<AppliedEffectResult> {
            if !e.is_charged_attack() || e.eff_state.get_usages() < 2 {
                return None;
            }

            cost.try_reduce_unaligned_cost(1)
                .then_some(AppliedEffectResult::NoChange)
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !e.is_charged_attack() {
                return None;
            }

            dmg.dmg += 1;
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod raging_oni_king {
    use super::*;

    pub const S: Status = Status::new_duration("Raging Oni King", StatusAttachMode::Character, 2);

    decl_status_impl_type!(RagingOniKing, I);
    impl StatusImpl for RagingOniKing {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillType::NormalAttack) = e.skill_type() else {
                return None;
            };
            dmg.dmg += 2;
            dmg.infuse(DealDMGType::GEO);
            Some(AppliedEffectResult::NoChange)
        }
    }
}
