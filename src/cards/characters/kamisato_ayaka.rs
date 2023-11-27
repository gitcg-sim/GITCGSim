use super::*;

pub const C: CharCard = CharCard {
    name: "Kamisato Ayaka",
    elem: Element::Cryo,
    weapon: WeaponType::Sword,
    faction: Faction::Inazuma,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::KamisatoArtKabuki,
        SkillId::KamisatoArtHyouka,
        SkillId::KamisatoArtSoumetsu,
    ],
    passive: Some(Passive::new("Kamisato Art: Senho").status(StatusId::KamisatoArtSenho)),
};

pub const KAMISATO_ART_KABUKI: Skill = skill_na("Kamisato Art: Kabuki", Element::Cryo, 2, DealDMGType::Physical);

pub const KAMISATO_ART_HYOUKA: Skill = Skill {
    name: "Kamisato Art: Hyouka",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Cryo, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 3, 0)),
    ..Skill::new()
};

pub const KAMISATO_ART_SOUMETSU: Skill = Skill {
    name: "Kamisato Art: Soumetsu",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Cryo, 3, 0, 3),
    deal_dmg: Some(deal_elem_dmg(Element::Cryo, 4, 0)),
    summon: Some(SummonSpec::One(SummonId::FrostflakeSekiNoTo)),
    ..Skill::new()
};

pub mod kamisato_art_senho {
    use super::*;

    pub const S: Status = Status::new_indef("Kamisato Art: Senho", StatusAttachMode::Character);

    decl_status_impl_type!(KamisatoArtSenho, I);
    impl StatusImpl for KamisatoArtSenho {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::Switched]
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            if e.c.is_switched_into_character(CharId::KamisatoAyaka) {
                e.out_cmds.push((
                    e.ctx_for_dmg.without_target(),
                    // TODO utility method to apply this command
                    Command::ApplyStatusToCharacter(
                        StatusId::CryoElementalInfusion,
                        e.c.ctx.src.dst_char_idx().unwrap(),
                    ),
                ));
            }
            Some(AppliedEffectResult::NoChange)
        }
    }
}

pub mod cryo_elemental_infusion {
    use super::*;

    pub const S: Status = Status::new_duration("Cryo Elemental Infusion", StatusAttachMode::Character, 1);

    decl_status_impl_type!(CryoElementalInfusion, I);
    impl StatusImpl for CryoElementalInfusion {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(SkillId::KamisatoArtKabuki) = e.skill_id() else {
                return None;
            };
            if !dmg.infuse(DealDMGType::CRYO) {
                return None;
            }
            if e.has_talent_equipped() && e.eff_state.can_use_once_per_round() {
                dmg.dmg += 1;
                Some(AppliedEffectResult::ConsumeOncePerRound)
            } else {
                Some(AppliedEffectResult::NoChange)
            }
        }
    }
}

pub mod frostflake_seki_no_to {
    use super::*;

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(DealDMG::new(DealDMGType::CRYO, 2, 0));

    pub const S: Status = Status::new_summon_usages("Frostflake Seki no To", 2);
}
