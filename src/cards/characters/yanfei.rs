use super::*;

pub const C: CharCard = CharCard {
    name: "Yanfei",
    elem: Element::Pyro,
    weapon: WeaponType::Catalyst,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 2,
    skills: list8![SkillId::SealOfApproval, SkillId::SignedEdict, SkillId::DoneDeal,],
    passive: None,
};

pub const SEAL_OF_APPROVAL: Skill = skill_na("Seal of Approval", Element::Pyro, 1, DealDMGType::PYRO);

pub const SIGNED_EDICT: Skill = Skill {
    name: "Signed Edict",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Pyro, 3, 0, 0),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 3, 0)),
    apply: Some(StatusId::ScarletSeal),
    ..Skill::new()
};

pub const DONE_DEAL: Skill = Skill {
    name: "Done Deal",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Pyro, 3, 0, 2),
    deal_dmg: Some(deal_elem_dmg(Element::Pyro, 3, 0)),
    skill_impl: Some(&DoneDeal()),
    ..Skill::new()
};

pub struct DoneDeal();

impl SkillImpl for DoneDeal {
    fn get_commands(&self, _: &PlayerState, ctx: &CommandContext, cmds: &mut CommandList<(CommandContext, Command)>) {
        cmds.push((*ctx, Command::ApplyStatusToActiveCharacter(StatusId::ScarletSeal)));
        cmds.push((*ctx, Command::ApplyStatusToActiveCharacter(StatusId::Brilliance)));
    }
}

pub mod scarlet_seal {
    use super::*;

    pub const S: Status = Status::new_usages("Scarlet Seal", StatusAttachMode::Character, 2, None);

    pub const I: IncreaseChargedAttackDMG = IncreaseChargedAttackDMG::new(2, AppliedEffectResult::ConsumeUsage);
}

pub mod brilliance {
    use super::*;

    pub const S: Status = Status::new_duration("Brilliance", StatusAttachMode::Character, 2);

    pub struct BrillianceUpdateCost();
    impl StatusImpl for BrillianceUpdateCost {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::UpdateCost]
        }

        fn update_cost(&self, e: &StatusImplContext, cost: &mut Cost, _: CostType) -> Option<AppliedEffectResult> {
            if !e.is_charged_attack() || !e.eff_state.can_use_once_per_round() {
                return None;
            }

            cost.try_reduce_elemental_cost(1, Element::Pyro)
                .then_some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }

    pub struct BrillianceAttachScarletSeal();
    trigger_event_impl!(BrillianceAttachScarletSeal, [EndPhase], |e| {
        e.out_cmds.push((
            *e.ctx_for_dmg,
            Command::ApplyStatusToCharacter(StatusId::ScarletSeal, e.status_key.char_idx().unwrap()),
        ));
        Some(AppliedEffectResult::NoChange)
    });

    compose_status_impls!(Brilliance(BrillianceUpdateCost(), BrillianceAttachScarletSeal()));
    decl_status_impl_type!(Brilliance, I);
}
