use crate::{card_impl_for_artifact, decl_status_impl_type};

use super::*;

macro_rules! artifact {
    ($name: expr, $cost: expr) => {
        use super::*;
        const NAME: &str = $name;

        pub const C: Card = Card {
            name: NAME,
            cost: $cost,
            card_type: CardType::Artifact,
            effects: list8![],
            card_impl: Some(&I),
        };
    };
}

macro_rules! artifact2 {
    ($name: expr, $elem: ident, $status_id: ident) => {
        artifact!($name, Cost::aligned(2));

        pub const S: Status = equipment_status(NAME);

        pub const I: ElementalArtifact = ElementalArtifact {
            elem: Element::$elem,
            status_id: StatusId::$status_id,
            dice_guarantee: None,
        };
    };
}

macro_rules! artifact3 {
    ($name: expr, $elem: ident, $status_id: ident) => {
        artifact!($name, Cost::aligned(3));

        pub const S: Status = equipment_status(NAME);

        pub const I: ElementalArtifact = ElementalArtifact {
            elem: Element::$elem,
            status_id: StatusId::$status_id,
            dice_guarantee: Some(2),
        };
    };
}

pub mod broken_rimes_echo {
    artifact2!("Broken Rime's Echo", Cryo, BrokenRimesEcho);
}

pub mod wine_stained_tricorne {
    artifact2!("Wine-Stained Tricorne", Hydro, WineStainedTricorne);
}

pub mod witchs_scorching_hat {
    artifact2!("Witch's Scorching Hat", Pyro, WitchsScorchingHat);
}

pub mod thunder_summoners_crown {
    artifact2!("Thunder Summoner's Crown", Electro, ThunderSummonersCrown);
}

pub mod viridescent_venerers_diadem {
    artifact2!("Viridescent Venerer's Diadem", Anemo, ViridescentVenerersDiadem);
}

pub mod mask_of_solitude_basalt {
    artifact2!("Mask of Solitude Basalt", Geo, MaskOfSolitudeBasalt);
}

pub mod laurel_coronet {
    artifact2!("Laurel Coronet", Dendro, LaurelCoronet);
}

pub mod blizzard_strayer {
    artifact3!("Blizzard Strayer", Cryo, BlizzardStrayer);
}

pub mod heart_of_depth {
    artifact3!("Heart of Depth", Hydro, HeartOfDepth);
}

pub mod crimson_witch_of_flames {
    artifact3!("Crimson Witch of Flames", Pyro, CrimsonWitchOfFlames);
}

pub mod thundering_fury {
    artifact3!("Thundering Fury", Electro, ThunderingFury);
}

pub mod viridescent_venerer {
    artifact3!("Viridescent Venerer", Anemo, ViridescentVenerer);
}

pub mod archaic_petra {
    artifact3!("Archaic Petra", Geo, ArchaicPetra);
}

pub mod deepwood_memories {
    artifact3!("Deepwood Memories", Dendro, DeepwoodMemories);
}

pub struct HealArtifact {
    pub status_id: StatusId,
    pub skill_type: SkillType,
    pub heal: u8,
    pub once_per_round: bool,
}

impl HealArtifact {
    pub const fn new(status_id: StatusId, skill_type: SkillType, heal: u8, once_per_round: bool) -> Self {
        Self {
            status_id,
            skill_type,
            heal,
            once_per_round,
        }
    }
}

impl StatusImpl for HealArtifact {
    fn responds_to(&self) -> EnumSet<RespondsTo> {
        enum_set![RespondsTo::TriggerXEvent]
    }

    fn responds_to_events(&self) -> XEventMask {
        xevent_mask::SKILL_FROM_SELF
            & match self.skill_type {
                SkillType::NormalAttack => xevent_mask::SKILL_NA,
                SkillType::ElementalSkill => xevent_mask::SKILL_SKILL,
                SkillType::ElementalBurst => xevent_mask::SKILL_BURST,
            }
    }

    fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
        let skill_type = e.get_event_skill_ensuring_attached_character()?.skill_type();
        if skill_type != self.skill_type {
            return None;
        }

        if self.once_per_round {
            if !e.c.eff_state.can_use_once_per_round() {
                return None;
            }

            e.add_cmd(Command::Heal(self.heal));
            return Some(AppliedEffectResult::ConsumeOncePerRound);
        }

        let c = e.c.eff_state.get_counter();
        if c == 0 {
            return None;
        }

        e.add_cmd(Command::Heal(self.heal));
        Some(AppliedEffectResult::SetCounter(c - 1))
    }
}

card_impl_for_artifact!(HealArtifact);

pub mod adventurers_bandana {
    artifact!("Adventurer's Bandana", Cost::ONE);

    pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Character).with_counter(CounterSpec {
        name: "Times per Round",
        default_value: 3,
        resets_at_turn_end: true,
    });

    pub const I: HealArtifact = HealArtifact::new(StatusId::AdventurersBandana, SkillType::NormalAttack, 1, false);
}

pub mod lucky_dogs_silver_circlet {
    artifact!("Lucky Dog's Silver Circlet", Cost::unaligned(2));

    pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Character);

    pub const I: HealArtifact = HealArtifact::new(StatusId::LuckyDogsSilverCirclet, SkillType::ElementalSkill, 2, true);
}

pub mod traveling_doctors_handkerchief {
    artifact!("Traveling Doctor's Handkerchief", Cost::ONE);

    pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Character);

    pub const I: HealArtifact = HealArtifact::new(
        StatusId::TravelingDoctorsHandkerchief,
        SkillType::ElementalBurst,
        1,
        true,
    );
}

pub mod gamblers_earrings {
    artifact!("Gambler's Earrings", Cost::ONE);

    pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Character);

    card_impl_for_artifact!(GamblersEarrings, StatusId::GamblersEarrings);
    decl_status_impl_type!(GamblersEarrings, I);
    impl StatusImpl for GamblersEarrings {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::DMG_OUTGOING & xevent_mask::DMG_DEFEAT
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let dmg = e.get_outgoing_dmg_ensuring_own_player()?;
            if !dmg.defeated {
                return None;
            }
            let equip_char_idx = e.c.status_key.char_idx()?;
            let active_char_idx = e.c.src_player_state.active_char_index;
            if equip_char_idx != active_char_idx {
                return None;
            }
            e.add_cmd(Command::AddDice(DiceCounter::omni(2)));
            Some(AppliedEffectResult::NoChange)
        }
    }
}

pub mod exiles_circlet {
    artifact!("Exile's Circlet", Cost::unaligned(2));

    pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Character);

    card_impl_for_artifact!(ExilesCirclet, StatusId::ExilesCirclet);
    decl_status_impl_type!(ExilesCirclet, I);
    impl StatusImpl for ExilesCirclet {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_BURST
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let SkillType::ElementalBurst = e.get_event_skill_ensuring_attached_character()?.skill_type() else {
                return None;
            };
            for (char_idx, _) in e.c.src_player_state.char_states.enumerate_valid() {
                if char_idx == e.active_char_idx() {
                    continue;
                }

                e.add_cmd(Command::AddEnergyToCharacter(1, char_idx));
            }
            None
        }
    }
}

pub mod ornate_kabuto {
    artifact!("Ornate Kabuto", Cost::unaligned(2));

    pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Character);

    card_impl_for_artifact!(OrnateKabuto, StatusId::OrnateKabuto);
    decl_status_impl_type!(OrnateKabuto, I);
    impl StatusImpl for OrnateKabuto {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_BURST
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let SkillType::ElementalBurst = e.get_event_skill_ensuring_own_player()?.skill_type() else {
                return None;
            };
            let Some(own_char_idx) = e.c.status_key.char_idx() else {
                return None;
            };
            if Some(own_char_idx) == e.c.ctx.src.char_idx() {
                return None;
            }

            e.add_cmd(Command::AddEnergyToCharacter(1, own_char_idx));
            None
        }
    }
}
