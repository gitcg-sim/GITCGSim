use super::*;

pub const C: CharCard = CharCard {
    name: "Fatui Pyro Agent",
    elem: Element::Pyro,
    weapon: WeaponType::Other,
    faction: Faction::Fatui,
    max_health: 10,
    max_energy: 2,
    skills: list8![SkillId::Thrust, SkillId::Prowl, SkillId::BladeAblaze,],
    passive: Some(Passive::new("Stealth Master").status(StatusId::Stealth)),
};

pub const THRUST: Skill = skill_na("Thrust", Element::Pyro, 2, DealDMGType::Physical);

pub const PROWL: Skill = Skill {
    name: "Prowl",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Pyro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 1, 0)),
    apply: Some(StatusId::Stealth),
    ..Skill::new()
};

pub const BLADE_ABLAZE: Skill = Skill {
    name: "Blade Ablaze",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Pyro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 5, 0)),
    ..Skill::new()
};

pub mod stealth {
    use super::*;

    pub const S: Status = Status::new_usages("Stealth", StatusAttachMode::Character, 2, None)
        .talent_usages_increase(CharId::FatuiPyroAgent, 1);

    decl_status_impl_type!(Stealth, I);
    impl StatusImpl for Stealth {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG | RespondsTo::OutgoingDMG]
        }

        fn incoming_dmg(&self, _: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.try_reduce(1, AppliedEffectResult::ConsumeUsage)
        }

        fn outgoing_dmg(&self, c: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.dmg += 1;
            if c.has_talent_equipped() {
                dmg.infuse(DealDMGType::PYRO);
            }
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }
}
