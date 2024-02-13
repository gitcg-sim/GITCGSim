use super::*;

pub const C: CharCard = CharCard {
    name: "Yae Miko",
    elem: Element::Electro,
    weapon: WeaponType::Catalyst,
    faction: Faction::Inazuma,
    max_health: 10,
    max_energy: 2,
    skills: list8![
        SkillId::SpiritfoxSinEater,
        SkillId::YakanEvocationSesshouSakura,
        SkillId::GreatSecretArtTenkoKenshin,
    ],
    passive: None,
};

pub const SPIRITFOX_SIN_EATER: Skill = skill_na("Spiritfox Sin-Eater", Element::Electro, 1, DealDMGType::ELECTRO);

pub const YAKAN_EVOCATION_SESSHOU_SAKURA: Skill = Skill {
    name: "Yakan Evocation: Sesshou Sakura",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Electro, 3, 0, 0),
    summon: Some(SummonSpec::One(SummonId::SesshouSakura)),
    ..Skill::new()
};

pub const GREAT_SECRET_ART_TENKO_KENSHIN: Skill = Skill {
    name: "Great Secret Art: Tenko Kenshin",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Electro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Electro, 4, 0)),
    skill_impl: Some(&GreatSecretArtTenkoKenshin()),
    ..Skill::new()
};

pub const SKILLS: [(SkillId, Skill); 3] = [
    (SkillId::SpiritfoxSinEater, SPIRITFOX_SIN_EATER),
    (SkillId::YakanEvocationSesshouSakura, YAKAN_EVOCATION_SESSHOU_SAKURA),
    (SkillId::GreatSecretArtTenkoKenshin, GREAT_SECRET_ART_TENKO_KENSHIN),
];

pub struct GreatSecretArtTenkoKenshin();
impl SkillImpl for GreatSecretArtTenkoKenshin {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        if !src_player.status_collection.has_summon(SummonId::SesshouSakura) {
            return;
        }
        cmds.push((*ctx, Command::DeleteStatus(StatusKey::Summon(SummonId::SesshouSakura))));
        cmds.push((*ctx, Command::ApplyStatusToTeam(StatusId::TenkoThunderbolts)));
    }
}

pub mod sesshou_sakura {
    use super::*;

    pub const S: Status = Status::new_usages("Sesshou Sakura", StatusAttachMode::Summon, 3, Some(6));

    pub struct SesshouSakuraEndOfRound();
    trigger_event_impl!(SesshouSakuraEndOfRound, [DeclareEndOfRound], |e| {
        if e.c.eff_state.get_usages() < 4 {
            return None;
        }

        e.cmd_deal_dmg(DealDMGType::ELECTRO, 1, 0);
        Some(AppliedEffectResult::ConsumeUsage)
    });

    compose_status_impls!(SesshouSakura(
        SesshouSakuraEndOfRound(),
        EndPhaseDealDMG(deal_elem_dmg(Element::Electro, 1, 0))
    ));
    decl_summon_impl_type!(SesshouSakura, I);
}

pub mod tenko_thunderbolts {
    use super::*;

    pub const S: Status = Status::new_usages("Tenko Thunderbolts", StatusAttachMode::Team, 1, None);

    decl_status_impl_type!(TenkoThunderbolts, I);
    trigger_event_impl!(TenkoThunderbolts, [BeforeAction], |e| {
        e.cmd_deal_dmg(DealDMGType::ELECTRO, 3, 0);
        Some(AppliedEffectResult::ConsumeUsage)
    });
}
