use super::*;

pub const C: CharCard = CharCard {
    name: "Raiden Shogun",
    elem: Element::Electro,
    weapon: WeaponType::Polearm,
    faction: Faction::Inazuma,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::Origin,
        SkillId::TranscendenceBalefulOmen,
        SkillId::SecretArtMusouShinsetsu,
    ],
    passive: Some(Passive {
        name: "Chakra Desiderata",
        apply_statuses: list8![StatusId::ChakraDesiderata],
    }),
};

pub const ORIGIN: Skill = skill_na("Origin", Element::Electro, 2, DealDMGType::Physical);

pub const TRANSCENDENCE_BALEFUL_OMEN: Skill = Skill {
    name: "Transcendence: Baleful Omen",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Electro, 3, 0, 0),
    summon: Some(SummonSpec::One(SummonId::EyeOfStormyJudgment)),
    ..Skill::new()
};

pub const SECRET_ART_MUSOU_SHINSETSU: Skill = Skill {
    name: "Secret Art: Musou Shinsetsu",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Electro, 4, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 3, 1)),
    commands: list8![Command::AddEnergyToNonActiveCharacters(2)],
    ..Skill::new()
};

pub mod eye_of_stormy_judgment {
    use super::*;

    pub const S: Status = Status::new_usages("Eye of Stormy Judgment", StatusAttachMode::Summon, 3, None);

    pub const I: EyeOfStormyJudgment = EyeOfStormyJudgment();

    pub struct EyeOfStormyJudgment();
    impl StatusImpl for EyeOfStormyJudgment {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillType::ElementalBurst) = e.skill_type() else {
                return None;
            };
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }

        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
            let EventId::EndPhase = e.event_id else { return None };
            e.cmd_deal_dmg(DealDMGType::ELECTRO, 1, 0);
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}

pub mod chakra_desiderata {
    use super::*;

    pub const S: Status =
        Status::new_indef("Chakra Desiderata", StatusAttachMode::Character).with_counter(CounterSpec {
            name: "Resolve",
            default_value: 0,
            resets_at_turn_end: false,
        });

    decl_status_impl_type!(ChakraDesiderata, I);
    impl StatusImpl for ChakraDesiderata {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_BURST
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillId::SecretArtMusouShinsetsu) = e.skill_id() else {
                return None;
            };
            let c = e.eff_state.get_counter();
            if c == 0 {
                return None;
            }
            dmg.dmg += if e.has_talent_equipped() { 2 * c } else { c };
            Some(AppliedEffectResult::SetCounter(0))
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let SkillType::ElementalBurst = e.get_event_skill_ensuring_own_player()?.skill_type() else {
                return None;
            };

            let char_idx_raiden_shogun = e.find_chararacter_for(|c| c.char_id == CharId::RaidenShogun);
            if e.src_char_idx() == char_idx_raiden_shogun.map(|t| t.0 as u8) {
                return None;
            }

            let c = e.c.eff_state.get_counter();
            if c >= 3 {
                None
            } else {
                Some(AppliedEffectResult::SetCounter(c + 1))
            }
        }
    }
}
