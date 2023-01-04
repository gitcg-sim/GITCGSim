use crate::data_structures::capped_list::CappedLengthList8;

use super::*;

pub const C: CharCard = CharCard {
    name: "Rhodeia of Loch",
    elem: Element::Hydro,
    weapon: WeaponType::Other,
    faction: Faction::Monster,
    max_health: 10,
    max_energy: 3,
    skills: list8![
        SkillId::Surge,
        SkillId::OceanidMimicSummoning,
        SkillId::TheMyriadWilds,
        SkillId::TideAndTorrent
    ],
    passive: None,
};

pub const SURGE: Skill = skill_na("Surge", Element::Hydro, 1, DealDMGType::HYDRO);

const SUMMON_IDS: CappedLengthList8<SummonId> = list8![
    SummonId::OceanidMimicSquirrel,
    SummonId::OceanidMimicRaptor,
    SummonId::OceanidMimicFrog
];

pub const OCEANID_MIMIC_SUMMONING: Skill = Skill {
    name: "Oceanid Mimic Summoning",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 3, 0, 0),
    summon: Some(SummonSpec::MultiRandom {
        summon_ids: SUMMON_IDS,
        count: 1,
        prioritize_new: true,
    }),
    ..Skill::new()
};

pub const THE_MYRIAD_WILDS: Skill = Skill {
    name: "The Myriad Wilds",
    skill_type: SkillType::ElementalSkill,
    cost: cost_elem(Element::Hydro, 5, 0, 0),
    summon: Some(SummonSpec::MultiRandom {
        summon_ids: SUMMON_IDS,
        count: 2,
        prioritize_new: true,
    }),
    ..Skill::new()
};

pub const TIDE_AND_TORRENT: Skill = Skill {
    name: "Tide and Torrent",
    skill_type: SkillType::ElementalBurst,
    cost: cost_elem(Element::Hydro, 3, 0, 3),
    skill_impl: Some(&TideAndTorrent()),
    ..Skill::new()
};

pub struct TideAndTorrent();
impl SkillImpl for TideAndTorrent {
    fn get_commands(
        &self,
        src_player: &PlayerState,
        ctx: &CommandContext,
        cmds: &mut CommandList<(CommandContext, Command)>,
    ) {
        let summon_count = src_player.status_collection.summon_count() as u8;
        let dmg = 2 + summon_count;
        cmds.push((*ctx, Command::DealDMG(DealDMG::new(DealDMGType::HYDRO, dmg, 0))));
    }
}

pub mod oceanid_mimic_squirrel {
    use super::*;

    pub const S: Status = Status::new_usages("Oceanid Mimic: Squirrel", StatusAttachMode::Summon, 2, None);

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(deal_elem_dmg(Element::Hydro, 2, 0));
}

pub mod oceanid_mimic_raptor {
    use super::*;

    pub const S: Status = Status::new_usages("Oceanid Mimic: Raptor", StatusAttachMode::Summon, 3, None);

    pub const I: EndPhaseDealDMG = EndPhaseDealDMG(deal_elem_dmg(Element::Hydro, 1, 0));
}

pub mod oceanid_mimic_frog {
    use super::*;

    pub const S: Status =
        Status::new_usages("Oceanid Mimic: Frog", StatusAttachMode::Summon, 2, None).with_manual_discard(true);

    pub const I: OceanidMimicFrog = OceanidMimicFrog();
    pub struct OceanidMimicFrog();
    impl StatusImpl for OceanidMimicFrog {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::IncomingDMG | RespondsTo::TriggerEvent]
        }

        fn incoming_dmg(&self, e: &StatusImplContext, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if e.eff_state.no_usages() {
                return None;
            }

            if dmg.reduce(1) {
                Some(AppliedEffectResult::ConsumeUsage)
            } else {
                None
            }
        }

        fn trigger_event(&self, e: &mut TriggerEventContext<EventId>) -> Option<AppliedEffectResult> {
            let EventId::EndPhase = e.event_id else { return None };
            if !e.c.eff_state.no_usages() {
                return None;
            }

            e.cmd_deal_dmg(DealDMGType::HYDRO, 2, 0);
            Some(AppliedEffectResult::DeleteSelf)
        }
    }
}
