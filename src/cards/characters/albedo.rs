use super::*;

pub const C: CharCard = CharCard {
    name: "Albedo",
    elem: Element::Geo,
    weapon: WeaponType::Sword,
    faction: Faction::Mondstadt,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::FavoniusBladeworkWeiss,
        SkillId::AbiogenesisSolarIsotoma,
        SkillId::RiteOfProgenitureTectonicTide,
    ],
    passive: None,
};

pub const FAVONIUS_BLADEWORK_WEISS: Skill =
    skill_na("Favonius Bladework - Weiss", Element::Geo, 2, DealDMGType::Physical);

pub const ABIOGENESIS_SOLAR_ISOTOMA: Skill = Skill {
    name: "Abiogenesis: Solar Isotoma",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Geo, 3, 0, 0),
    summon: Some(SummonSpec::One(SummonId::SolarIsotoma)),
    ..Skill::new()
};

pub const RITE_OF_PROGENITURE_TECTONIC_TIDE: Skill = Skill {
    name: "Rite of Progeniture: Tectonic Tide",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Geo, 3, 0, 2),
    skill_impl: Some(&RiteOfProgenitureTectonicTide()),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::FavoniusBladeworkWeiss, FAVONIUS_BLADEWORK_WEISS),
    (SkillId::AbiogenesisSolarIsotoma, ABIOGENESIS_SOLAR_ISOTOMA),
    (
        SkillId::RiteOfProgenitureTectonicTide,
        RITE_OF_PROGENITURE_TECTONIC_TIDE,
    ),
];

pub struct RiteOfProgenitureTectonicTide();
impl SkillImpl for RiteOfProgenitureTectonicTide {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        let dmg = if src_player.status_collection.has_summon(SummonId::SolarIsotoma) {
            6
        } else {
            4
        };
        cmds.push((*ctx, Command::DealDMG(DealDMG::new(DealDMGType::GEO, dmg, 0))));
    }
}

pub mod solar_isotoma {
    use crate::increase_outgoing_dmg_impl;

    use super::*;

    pub const S: Status =
        Status::new_usages("Solar Isotoma", StatusAttachMode::Summon, 3, None).casted_by_character(CharId::Albedo);

    pub struct SolarIsotomaReducePlungingAttackCost();
    impl StatusImpl for SolarIsotomaReducePlungingAttackCost {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::UpdateCost]
        }

        fn update_cost(&self, e: &StatusImplContext, cost: &mut Cost, _: CostType) -> Option<AppliedEffectResult> {
            if !e.is_plunging_attack() {
                return None;
            }

            cost.try_reduce_unaligned_cost(1)
                .then_some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }

    pub struct SolarIsotomaUnderTalentIncreasePlungingAttackDMG {
        pub dmg_increase: u8,
        pub result: AppliedEffectResult,
    }
    increase_outgoing_dmg_impl!(SolarIsotomaUnderTalentIncreasePlungingAttackDMG, |e, _dmg| e
        .has_talent_equipped()
        && e.is_plunging_attack());

    decl_summon_impl_type!(SolarIsotoma, I);
    compose_status_impls!(SolarIsotoma(
        EndPhaseDealDMG(deal_elem_dmg(Element::Geo, 1, 0)),
        SolarIsotomaUnderTalentIncreasePlungingAttackDMG {
            dmg_increase: 1,
            result: AppliedEffectResult::NoChange
        },
        SolarIsotomaReducePlungingAttackCost(),
    ));
}
