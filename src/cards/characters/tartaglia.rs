use super::*;

pub const C: CharCard = CharCard {
    name: "Tartaglia",
    elem: Element::Hydro,
    weapon: WeaponType::Bow,
    faction: Faction::Fatui,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::CuttingTorrent,
        SkillId::FoulLegacyRagingTide,
        SkillId::HavocObliteration,
    ],
    passive: Some(Passive {
        name: "Tide Withholder",
        apply_statuses: list8![StatusId::RangedStance],
    }),
};

pub const CUTTING_TORRENT: Skill = skill_na("Cutting Torrent", Element::Hydro, 2, DealDMGType::Physical);

pub const FOUL_LEGACY_RAGING_TIDE: Skill = Skill {
    name: "Foul Legacy: Raging Tide",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    skill_impl: Some(&FoulLegacyRagingTide()),
    deal_dmg: Some(deal_elem_dmg(Element::Hydro, 2, 0)),
    ..Skill::new()
};

pub const HAVOC_OBLITERATION: Skill = Skill {
    name: "Havoc: Obliteration",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 3),
    skill_impl: Some(&HavocObliteration()),
    ..Skill::new()
};

impl PlayerState {
    #[inline]
    fn is_melee_stance(&self) -> bool {
        self.status_collection
            .has_character_status(self.active_char_idx, StatusId::MeleeStance)
    }
}

struct FoulLegacyRagingTide();
impl SkillImpl for FoulLegacyRagingTide {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        if src_player.is_melee_stance() {
            return;
        }
        let char_idx = src_player.active_char_idx;
        cmds.push((
            *ctx,
            Command::DeleteStatus(StatusKey::Character(char_idx, StatusId::RangedStance)),
        ));
        cmds.push((*ctx, Command::ApplyStatusToCharacter(StatusId::MeleeStance, char_idx)));
    }
}

struct HavocObliteration();
impl SkillImpl for HavocObliteration {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        if src_player.is_melee_stance() {
            cmds.push((*ctx, Command::DealDMG(deal_elem_dmg(Element::Hydro, 7, 0))));
        } else {
            cmds.push((*ctx, Command::DealDMG(deal_elem_dmg(Element::Hydro, 4, 0))));
            cmds.push((*ctx, Command::AddEnergy(2)));
            cmds.push((*ctx, Command::ApplyStatusToTarget(StatusId::Riptide)));
        }
    }
}

pub mod riptide {
    use super::*;

    pub const S: Status = Status::new_duration("Riptide", StatusAttachMode::Character, 2)
        .with_applies_to_opposing()
        .with_shifts_to_next_active_on_death();

    decl_status_impl_type!(Riptide, I);
    impl StatusImpl for Riptide {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG]
        }

        fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            let Some(CharId::Tartaglia) = e.src_char_id() else {
                return None;
            };
            dmg.dmg += 2;
            Some(AppliedEffectResult::NoChange)
        }

        // TODO apply to active character when defeated
    }
}

pub mod melee_stance {
    use super::*;

    pub const S: Status = Status::new_duration("Melee Stance", StatusAttachMode::Character, 2)
        .with_reapplies_on_discard(StatusId::RangedStance);

    pub struct MeleeStanceOutgoingDMG();
    impl StatusImpl for MeleeStanceOutgoingDMG {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, c: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.infuse(DealDMGType::HYDRO);
            if c.dmg.target_affected_by_riptide {
                dmg.dmg += 1;
            }
            Some(AppliedEffectResult::NoChange)
        }
    }

    pub struct MeleeStanceNA();
    impl AttachedCharacterSkillEvent for MeleeStanceNA {
        const SKILL_TYPES: EnumSet<SkillType> = enum_set![SkillType::NormalAttack];
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            if !e.c.is_charged_attack() {
                return None;
            }
            e.add_cmd(Command::ApplyStatusToTarget(StatusId::Riptide));
            Some(AppliedEffectResult::NoChange)
        }
    }

    compose_status_impls!(MeleeStance(
        MeleeStanceOutgoingDMG(),
        AttachedCharacterSkillEventI(MeleeStanceNA())
    ));
    decl_status_impl_type!(MeleeStance, I);
}

pub mod ranged_stance {
    use super::*;

    pub const S: Status = Status::new_indef("Ranged Stance", StatusAttachMode::Character);

    decl_event_handler_trait_impl!(OwnCharacterSkillEvent(RangedStance), I);
    impl OwnCharacterSkillEvent for RangedStance {
        const SKILL_TYPES: EnumSet<SkillType> = enum_set![SkillType::NormalAttack];
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            if !e.c.is_charged_attack() {
                return None;
            }
            e.add_cmd(Command::ApplyStatusToTarget(StatusId::Riptide));
            Some(AppliedEffectResult::NoChange)
        }
    }
}
