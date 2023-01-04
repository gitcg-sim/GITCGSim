use super::*;

pub const C: CharCard = CharCard {
    name: "Amber",
    elem: Element::Pyro,
    weapon: WeaponType::Bow,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 2,
    skills: list8![SkillId::Sharpshooter, SkillId::ExplosivePuppet, SkillId::FieryRain],
    passive: None,
};

pub const SHARPSHOOTER: Skill = skill_na("Sharpshooter", Element::Pyro, 2, DealDMGType::Physical);

pub const EXPLOSIVE_PUPPET: Skill = Skill {
    name: "Explosive Puppet",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Pyro, 3, 0, 0),
    ..Skill::new()
};

pub const FIERY_RAIN: Skill = Skill {
    name: "Fiery Rain",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Pyro, 3, 0, 2),
    ..Skill::new()
};

pub mod baron_bunny {
    use super::*;

    pub const S: Status =
        Status::new_usages("Baron Bunny", StatusAttachMode::Summon, 1, None).with_manual_discard(true);

    pub const I: BaronBunny = BaronBunny();
    pub struct BaronBunny();
    impl StatusImpl for BaronBunny {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerEvent | RespondsTo::IncomingDMG]
        }

        fn responds_to_triggers(&self) -> EnumSet<EventId> {
            enum_set![EventId::EndPhase]
        }

        fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if e.eff_state.get_usages() == 0 {
                return None;
            }
            dmg.reduce(2);
            Some(AppliedEffectResult::ConsumeUsage)
        }

        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
            e.cmd_deal_dmg(DealDMGType::PYRO, 2, 0);
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}
