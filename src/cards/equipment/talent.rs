use super::*;

macro_rules! talent {
    (@skill $skill: ident) => { Some(SkillId::$skill) };
    (@skill) => { None };
    (@status $status: ident) => { Some(StatusId::$status) };
    (@status) => { None };
    (
        $ident: ident,
        $name: expr, $cost: expr, $char: ident
        $(, skill => $skill: ident)?
        $(, status => $status: ident)?
        $(,)?
    ) => {
        use super::*;
        const NAME: &str = $name;

        pub const C: Card = Card {
            name: NAME,
            cost: $cost,
            card_type: CardType::Talent(CharId::$char),
            effects: list8![],
            card_impl: Some(&$ident::TALENT),
        };

        pub struct $ident();

        impl $ident {
            pub const TALENT: Talent = Talent {
                skill_id: talent!(@skill $($skill)?),
                status_id: talent!(@status $($status)?),
            };
        }

        pub const I: $ident = $ident();

        $(
            pub const S: Status = Status::new_indef(NAME, StatusAttachMode::Character);
            impl $ident {
                #[allow(dead_code)]
                pub const STATUS_ID: StatusId = StatusId::$status;
            }
        )?
    };
}

pub mod naganohara_meteor_swarm {
    talent!(
        NaganoharaMeteorSwarm,
        "Naganohara Meteor Swarm",
        Cost::elem(Element::Pyro, 2),
        Yoimiya,
        skill => NiwabiFireDance
    );
}

pub mod kanten_senmyou_blessing {
    talent!(
        KantenSenmyouBlessing,
        "Kanten Senmyou Blessing",
        Cost::elem(Element::Cryo, 2),
        KamisatoAyaka
    );
}

pub mod i_got_your_back {
    talent!(
        IGotYourBack,
        "I Got Your Back",
        Cost::elem(Element::Geo, 3),
        Noelle,
        skill => Breastplate
    );
}

pub mod crossfire {
    talent!(
        Crossfire,
        "Crossfire",
        Cost::elem(Element::Pyro, 4),
        Xiangling,
        skill => GuobaAttack
    );
}

pub mod floral_sidewinder {
    talent!(
        FloralSidewinder,
        "Floral Sidewinder",
        Cost::elem(Element::Dendro, 4),
        Collei,
        skill => FloralBrush
    );
}

pub mod flowing_flame {
    talent!(
        FlowingFlame,
        "Flowing Flame",
        Cost::elem(Element::Pyro, 3),
        Diluc,
        skill => SearingOnslaught
    );
}

pub mod prophecy_of_submersion {
    talent!(
        ProphecyOfSubmersion,
        "Prophecy of Submersion",
        Cost::elem(Element::Hydro, 3).with_energy(3),
        Mona,
        skill => StellarisPhantasm
    );
}

pub mod strategic_reserve {
    talent!(
        StrategicReserve,
        "Strategic Reserve",
        Cost::elem(Element::Geo, 4),
        Ningguang,
        skill => JadeScreen
    );
}

pub mod lands_of_dandelion {
    talent!(
        LandsOfDandelion,
        "Lands of Dandelion",
        Cost::elem(Element::Anemo, 4).with_energy(3),
        Jean,
        skill => DandelionBreeze,
    );
}

pub mod the_scent_remained {
    talent!(
        TheScentRemained,
        "The Scent Remained",
        Cost::elem(Element::Hydro, 4),
        Xingqiu,
        skill => FatalRainscreen,
    );
}

pub mod pounding_surprise {
    talent!(
        PoundingSurprise,
        "Pounding Surprise",
        Cost::elem(Element::Pyro, 3),
        Klee,
        skill => JumpyDumpty,
    );
}

pub mod thundering_penance {
    talent!(
        ThunderingPenance,
        "Thundering Penance",
        Cost::elem(Element::Electro, 3),
        Keqing,
        skill => StellarRestoration,
    );
}

pub mod paid_in_full {
    talent!(
        PaidInFull,
        "Paid in Full",
        Cost::elem(Element::Pyro, 3),
        FatuiPyroAgent,
        skill => Prowl,
    );
}

pub mod steady_breathing {
    talent!(
        SteadyBreathing,
        "Steady Breathing",
        Cost::elem(Element::Cryo, 4),
        Chongyun,
        skill => ChonghuasLayeredFrost,
    );
}

pub mod grand_expectation {
    talent!(
        GrandExpectation,
        "Grand Expectation",
        Cost::elem(Element::Pyro, 4).with_energy(2),
        Bennett,
        skill => FantasticVoyage,
    );
}

pub mod glorious_season {
    talent!(
        GloriousSeason,
        "Glorious Season",
        Cost::elem(Element::Hydro, 4),
        Barbara,
        skill => LetTheShowBegin,
    );
}

pub mod shaken_not_purred {
    talent!(
        ShakenNotPurred,
        "Shaken, Not Purred",
        Cost::elem(Element::Cryo, 4),
        Diona,
        skill => IcyPaws,
    );
}

pub mod cold_blooded_strike {
    talent!(
        ColdBloodedStrike,
        "Cold-Blooded Strike",
        Cost::elem(Element::Cryo, 4),
        Kaeya,
        skill => Frostgnaw,
        status => ColdBloodedStrike,
    );

    impl StatusImpl for ColdBloodedStrike {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::TriggerXEvent]
        }

        fn responds_to_events(&self) -> XEventMask {
            xevent_mask::SKILL_FROM_SELF & xevent_mask::SKILL_SKILL
        }

        fn trigger_xevent(&self, e: &mut TriggerEventContext<XEvent>) -> Option<AppliedEffectResult> {
            let SkillId::Frostgnaw = e.get_event_skill_ensuring_attached_character()?.skill_id else {
                return None;
            };
            if !e.c.eff_state.can_use_once_per_round() {
                return None;
            }
            e.add_cmd(Command::Heal(2));
            Some(AppliedEffectResult::ConsumeOncePerRound)
        }
    }
}

pub mod undivided_heart {
    talent!(
        UndividedHeart,
        "Undivided Heart",
        Cost::elem(Element::Cryo, 5),
        Ganyu,
        skill => FrostflakeArrow,
        status => UndividedHeart,
    );

    impl StatusImpl for UndividedHeart {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !e.src_player_state.flags.contains(PlayerFlag::SkillCastedThisMatch) {
                return None;
            }
            let Some(SkillId::FrostflakeArrow) = e.skill_id() else {
                return None;
            };
            dmg.dmg += 1;
            dmg.piercing_dmg_to_standby = 3;
            Some(AppliedEffectResult::NoChange)
        }
    }
}

pub mod kyouka_fuushi {
    talent!(
        KyoukaFuushi,
        "Kyouka Fuushi",
        Cost::elem(Element::Hydro, 3),
        KamisatoAyato,
        skill => KamisatoArtKyouka,
    );
}

pub mod wishes_unnumbered {
    talent!(
        WishesUnnumbered,
        "Wishes Unnumbered",
        Cost::elem(Element::Electro, 4).with_energy(2),
        RaidenShogun,
        skill => SecretArtMusouShinsetsu,
    );
}

pub mod keen_sight {
    talent!(
        KeenSight,
        "Keen Sight",
        Cost::elem(Element::Dendro, 4),
        Tighnari,
        skill => VijnanaPhalaMine,
    );
}

pub mod sanguine_rouge {
    talent!(
        SanguineRouge,
        "Sanguine Rouge",
        Cost::elem(Element::Pyro, 2),
        HuTao,
        skill => GuideToAfterlife,
    );
}

pub mod right_of_final_interpretation {
    talent!(
        RightOfFinalInterpretation,
        "Right of Final Interpretation",
        Cost::elem(Element::Pyro, 1).with_unaligned(2),
        Yanfei,
        skill => SealOfApproval,
        status => RightOfFinalInterpretation,
    );

    impl StatusImpl for RightOfFinalInterpretation {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if !e.is_charged_attack() || e.dmg.target_hp > 6 {
                return None;
            }
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }
    }
}

pub mod mystical_abandon {
    talent!(
        MysticalAbandon,
        "Mystical Abandon",
        Cost::elem(Element::Cryo, 3),
        Shenhe,
        skill => SpringSpiritSummoning,
    );
}

pub mod the_overflow {
    talent!(
        TheOverflow,
        "The Overflow",
        Cost::elem(Element::Hydro, 4).with_energy(2),
        Candace,
        skill => SacredRiteWagtailsTide,
    );
}
