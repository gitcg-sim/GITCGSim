use super::*;

pub const C: CharCard = CharCard {
    name: "Qiqi",
    elem: Element::Cryo,
    weapon: WeaponType::Sword,
    faction: Faction::Liyue,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::AncientSwordArt,
        SkillId::AdeptusArtHeraldOfFrost,
        SkillId::AdeptusArtPreserverOfFortune,
    ],
    passive: None,
};

pub const ANCIENT_SWORD_ART: Skill = skill_na("Ancient Sword Art", Element::Cryo, 2, DealDMGType::Physical);

pub const ADEPTUS_ART_HERALD_OF_FROST: Skill = Skill {
    name: "Adeptus Art: Herald of Frost",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Cryo, 3, 0, 0),
    ..Skill::new()
};

pub const ADEPTUS_ART_PRESERVER_OF_FORTUNE: Skill = Skill {
    name: "Adeptus Art: Preserver of Fortune",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Cryo, 3, 0, 3),
    ..Skill::new()
};

pub mod herald_of_frost {
    use super::*;

    pub const S: Status = Status::new_usages("Herald of Frost", StatusAttachMode::Summon, 3, None);

    pub struct HeraldOfFrostHealEffect();
    impl OwnCharacterSkillEvent for HeraldOfFrostHealEffect {
        const SKILL_TYPES: EnumSet<SkillType> = enum_set![SkillType::NormalAttack];
        fn invoke(c: &mut TriggerEventContext<XEvent>, evt: XEventSkill) -> Option<AppliedEffectResult> {
            let SkillId::AncientSwordArt = evt.skill_id else {
                return None;
            };

            // TODO doesn't work
            c.add_cmd(Command::Heal(1));

            Some(AppliedEffectResult::NoChange)
        }
    }

    decl_summon_impl_type!(HeraldOfFrost, I);
    compose_status_impls!(HeraldOfFrost(
        EndPhaseDealDMG(deal_elem_dmg(Element::Cryo, 1, 0)),
        OwnCharacterSkillEventI(HeraldOfFrostHealEffect())
    ));
}

pub mod fortune_preserving_talisman {
    use super::*;

    pub const S: Status = Status::new_usages("Fortune-Preserving Talisman", StatusAttachMode::Team, 3, None);

    pub struct FortunePreservingTalisman();
    impl OwnCharacterSkillEvent for FortunePreservingTalisman {
        const SKILL_TYPES: EnumSet<SkillType> =
            enum_set![SkillType::NormalAttack | SkillType::ElementalSkill | SkillType::ElementalBurst];
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            if e.c.src_player_state.active_char_state().is_max_hp() {
                return None;
            }

            e.add_cmd(Command::Heal(2));
            Some(AppliedEffectResult::ConsumeUsage)
        }
    }

    pub const I: OwnCharacterSkillEventI<FortunePreservingTalisman> =
        OwnCharacterSkillEventI(FortunePreservingTalisman());
}
