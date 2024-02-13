use super::*;

pub const C: CharCard = CharCard {
    name: "Jadeplume Terrorshroom",
    elem: Element::Dendro,
    weapon: WeaponType::Other,
    faction: Faction::Monster,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::MajesticDance,
        SkillId::VolatileSporeCloud,
        SkillId::FeatherSpreading,
    ],
    passive: Some(Passive::new("Radical Vitality").status(StatusId::RadicalVitality)),
};

pub const MAJESTIC_DANCE: Skill = skill_na("Majestic Dance", Element::Dendro, 2, DealDMGType::Physical);

pub const VOLATILE_SPORE_CLOUD: Skill = Skill {
    name: "Volatile Spore Cloud",
    skill_type: SkillType::ElementalSkill,
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 3, 0)),
    cost: cost_elem(Element::Dendro, 3, 0, 0),
    ..Skill::new()
};

pub const FEATHER_SPREADING: Skill = Skill {
    name: "Feather Spreading",
    skill_type: SkillType::ElementalBurst,
    deal_dmg: Some(deal_elem_dmg(Element::Dendro, 4, 0)),
    cost: cost_elem(Element::Dendro, 3, 0, 2),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::MajesticDance, MAJESTIC_DANCE),
    (SkillId::VolatileSporeCloud, VOLATILE_SPORE_CLOUD),
    (SkillId::FeatherSpreading, FEATHER_SPREADING),
];

pub mod radical_vitality {
    use super::*;

    pub const S: Status = Status::new_indef("Radical Vitality", StatusAttachMode::Character)
        .counter(CounterSpec::new("Radical Vitality", 0));

    #[inline]
    fn increase_stacks(s: u8) -> u8 {
        if s >= 3 {
            3
        } else {
            s + 1
        }
    }

    decl_status_impl_type!(RadicalVitality, I);
    impl StatusImpl for RadicalVitality {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent | RespondsTo::IncomingDMG | RespondsTo::OutgoingDMG]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !dmg.dmg_type.is_elemental() {
                return None;
            }

            let stacks = e.eff_state.get_counter();
            Some(AppliedEffectResult::SetCounter(increase_stacks(stacks)))
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !dmg.dmg_type.is_elemental() {
                return None;
            }

            let stacks = e.eff_state.get_counter();
            if let Some(SkillId::FeatherSpreading) = e.skill_id() {
                // 2 stacks when casting -> 4 + 2 = 6 DMG from Burst
                dmg.dmg += stacks;
                Some(AppliedEffectResult::SetCounter(0))
            } else {
                Some(AppliedEffectResult::SetCounter(increase_stacks(stacks)))
            }
        }

        fn trigger_event(&self, e: &mut TriggerEventContext) -> Option<AppliedEffectResult> {
            let stacks = e.c.eff_state.get_counter();
            match e.event_id {
                EventId::EndPhase => {
                    if stacks >= 3 {
                        e.out_cmds
                            .push((*e.ctx_for_dmg, Command::SetEnergyForActiveCharacter(0)));
                        Some(AppliedEffectResult::SetCounter(0))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }
}
