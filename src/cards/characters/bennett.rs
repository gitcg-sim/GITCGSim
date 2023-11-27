use super::*;

pub const C: CharCard = CharCard {
    name: "Bennett",
    elem: Element::Pyro,
    weapon: WeaponType::Sword,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::StrikeOfFortune,
        SkillId::PassionOverload,
        SkillId::FantasticVoyage,
    ],
    passive: None,
};

pub const STRIKE_OF_FORTUNE: Skill = skill_na("Strike of Fortune", Element::Pyro, 2, DealDMGType::Physical);

pub const PASSION_OVERLOAD: Skill = Skill {
    name: "Passion Overload",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Pyro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 3, 0)),
    ..Skill::new()
};

pub const FANTASTIC_VOYAGE: Skill = Skill {
    name: "Fantastic Voyage",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Pyro, 4, 0, 2),
    apply: Some(StatusId::InspirationField),
    ..Skill::new()
};

pub mod inspiration_field {
    use super::*;

    pub const S: Status =
        Status::new_duration("Inspiration Field", StatusAttachMode::Team, 2).casted_by_character(CharId::Bennett);

    pub struct InspirationFieldOutgoingDMG();
    impl StatusImpl for InspirationFieldOutgoingDMG {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(cs) = e.get_src_character_state() else {
                return None;
            };
            if e.has_talent_equipped() || cs.get_hp() >= 7 {
                dmg.dmg += 2;
                Some(AppliedEffectResult::NoChange)
            } else {
                None
            }
        }
    }

    pub struct InspirationFieldEvent();
    impl OwnCharacterSkillEvent for InspirationFieldEvent {
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            let Some(cs) = e.c.get_src_character_state() else {
                return None;
            };
            if cs.get_hp() <= 6 {
                e.out_cmds.push((*e.ctx_for_dmg, Command::Heal(2)));
            }
            Some(AppliedEffectResult::NoChange)
        }
    }

    compose_status_impls!(InspirationField(
        InspirationFieldOutgoingDMG(),
        OwnCharacterSkillEventI(InspirationFieldEvent())
    ));
    decl_status_impl_type!(InspirationField, I);
}
