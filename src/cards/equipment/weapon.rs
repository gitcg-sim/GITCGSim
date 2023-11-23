use super::*;
use crate::card_impl_for_weapon;

macro_rules! weapon {
    (
        $impl: ident, $name: expr, $type: ident, $status_id: ident,
        $cost: expr
        $(, effects => $effects: expr)?
        $(, status => $status: expr)? $(,)?
    ) => {
        use super::*;
        pub const NAME: &str = $name;
        pub const C: Card = Card {
            name: NAME,
            cost: $cost,
            card_type: CardType::Weapon(WeaponType::$type),
            effects: weapon!(@effects $($effects)?),
            card_impl: Some(&I),
        };

        pub const S: Status = weapon!(@status $($status)?);

        pub const I: $impl = $impl {
            weapon_type: WeaponType::$type,
            status_id: StatusId::$status_id,
        };
    };
    (@status $(,)?) => { equipment_status(NAME) };
    (@status $expr: expr) => { $expr };
    (@effects $(,)?) => { list8![] };
    (@effects $expr: expr) => { $expr };
}

macro_rules! weapon2 {
    ($name: expr, $type: ident, $status_id: ident) => {
        weapon!(Weapon2, $name, $type, $status_id, Cost::aligned(2));
    };
}

pub mod magic_guide {
    weapon2!("Magic Guide", Catalyst, MagicGuide);
}

pub mod raven_bow {
    weapon2!("Raven Bow", Bow, RavenBow);
}

pub mod white_iron_greatsword {
    weapon2!("White Iron Greatsword", Claymore, WhiteIronGreatsword);
}

pub mod white_tassel {
    weapon2!("White Tassel", Polearm, WhiteTassel);
}

pub mod travelers_handy_sword {
    weapon2!("Traveler's Handy Sword", Sword, TravelersHandySword);
}

pub mod sacrificial_fragments {
    weapon!(
        SacrificialWeapon,
        "Sacrificial Fragments",
        Catalyst,
        SacrificialFragments,
        Cost::aligned(3)
    );
}

pub mod sacrificial_bow {
    weapon!(
        SacrificialWeapon,
        "Sacrificial Bow",
        Bow,
        SacrificialBow,
        Cost::aligned(3)
    );
}

pub mod sacrificial_greatsword {
    weapon!(
        SacrificialWeapon,
        "Sacrificial Greatsword",
        Claymore,
        SacrificialGreatsword,
        Cost::aligned(3)
    );
}

pub mod sacrificial_sword {
    weapon!(
        SacrificialWeapon,
        "Sacrificial Sword",
        Sword,
        SacrificialSword,
        Cost::aligned(3)
    );
}

pub mod skyward_atlas {
    weapon!(SkywardWeapon, "Skyward Atlas", Sword, SkywardAtlas, Cost::aligned(3));
}

pub mod skyward_harp {
    weapon!(SkywardWeapon, "Skyward Harp", Bow, SkywardHarp, Cost::aligned(3));
}

pub mod skyward_spine {
    weapon!(SkywardWeapon, "Skyward Spine", Polearm, SkywardSpine, Cost::aligned(3));
}

pub mod skyward_pride {
    weapon!(SkywardWeapon, "Skyward Pride", Claymore, SkywardPride, Cost::aligned(3));
}

pub mod wolfs_gravestone {
    pub struct WolfsGravestoneWeapon {
        pub weapon_type: WeaponType,
        pub status_id: StatusId,
    }

    impl StatusImpl for WolfsGravestoneWeapon {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if e.dmg.target_hp <= 6 {
                dmg.dmg += 3;
            } else {
                dmg.dmg += 1;
            }
            Some(AppliedEffectResult::NoChange)
        }
    }

    card_impl_for_weapon!(WolfsGravestoneWeapon);

    weapon!(
        WolfsGravestoneWeapon,
        "Wolf's Gravestone",
        Claymore,
        WolfsGravestone,
        Cost::aligned(3)
    );
}

pub mod lithic_spear {
    pub struct LithicSpearWeapon {
        pub weapon_type: WeaponType,
        pub status_id: StatusId,
    }

    impl StatusImpl for LithicSpearWeapon {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, _: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }
    }

    impl CardImpl for LithicSpearWeapon {
        fn can_be_played(&self, cic: &CardImplContext) -> CanBePlayedResult {
            can_be_played_for_weapon(self.weapon_type, cic)
        }

        fn selection(&self) -> Option<CardSelectionSpec> {
            Some(CardSelectionSpec::OwnCharacter)
        }

        fn get_effects(
            &self,
            cic: &CardImplContext,
            ctx: &CommandContext,
            commands: &mut CommandList<(CommandContext, Command)>,
        ) {
            get_effects_for_weapon(self.status_id, cic, ctx, commands);
            let Some(CardSelection::OwnCharacter(i)) = cic.selection else {
                unreachable!()
            };
            let src_player = &cic.game_state.players[ctx.src_player_id];
            let n = src_player
                .char_states
                .iter_valid()
                .filter(|c| c.char_id.get_char_card().faction == Faction::Liyue)
                .count() as u8;
            commands.push((
                *ctx,
                Command::IncreaseStatusUsages(
                    StatusKey::Equipment(i, EquipSlot::Weapon, StatusId::LithicSpear),
                    std::cmp::min(n, 3),
                ),
            ))
        }
    }

    weapon!(
        LithicSpearWeapon,
        "Lithic Spear",
        Polearm,
        LithicSpear,
        Cost::aligned(3),
        status => equipment_status(NAME).with_shield_points(0)
    );
}

pub mod favonius_sword {
    weapon!(FavoniusWeapon, "Favonius Sword", Sword, FavoniusSword, Cost::aligned(3));
}

/// This card does not exist in the Genius Invokation TCG.
///
/// **"Rust"**
/// Weapon/Bow, Cost: 3 matching
///
/// This character's Normal Attack deal +2 additional DMG, but this character's Charged Attack deals 1 less DMG.
///
/// (Charged Attack: When the total number of Elemental Dice is even, the Normal Attack to use will be considered a Charged Attack.)
///
/// (Only Bow Characters can equip this. A character can equip a maximum of 1 Weapon.)
pub mod rust {
    pub struct RustWeapon {
        pub weapon_type: WeaponType,
        pub status_id: StatusId,
    }

    impl StatusImpl for RustWeapon {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG]
        }

        fn outgoing_dmg(&self, e: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            if e.is_charged_attack() {
                dmg.try_reduce(1, AppliedEffectResult::NoChange)
            } else {
                dmg.dmg += 2;
                Some(AppliedEffectResult::NoChange)
            }
        }
    }

    card_impl_for_weapon!(RustWeapon);

    weapon!(RustWeapon, "Rust", Bow, Rust, Cost::aligned(3));
}

pub mod a_thousand_floating_dreams {
    pub struct AThousandFloatingDreamsWeapon {
        pub weapon_type: WeaponType,
        pub status_id: StatusId,
    }

    impl StatusImpl for AThousandFloatingDreamsWeapon {
        fn responds_to(&self) -> EnumSet<RespondsTo> {
            enum_set![RespondsTo::OutgoingDMG | RespondsTo::OutgoingReactionDMG]
        }

        fn outgoing_reaction_dmg(
            &self,
            e: &StatusImplContext<DMGInfo>,
            _: (Reaction, Option<Element>),
            dmg: &mut DealDMG,
        ) -> Option<AppliedEffectResult> {
            let c = e.eff_state.get_counter();
            if c == 0 {
                None
            } else {
                dmg.dmg += 1;
                Some(AppliedEffectResult::SetCounter(c - 1))
            }
        }

        fn outgoing_dmg(&self, _: &StatusImplContext<DMGInfo>, dmg: &mut DealDMG) -> Option<AppliedEffectResult> {
            dmg.dmg += 1;
            Some(AppliedEffectResult::NoChange)
        }
    }

    card_impl_for_weapon!(AThousandFloatingDreamsWeapon);

    weapon!(
        AThousandFloatingDreamsWeapon,
        "A Thousand Floating Dreams",
        Catalyst,
        AThousandFloatingDreams,
        Cost::aligned(3),
        status => equipment_status(NAME).with_counter(CounterSpec::new("[Counter]", 2).resets_at_turn_end(true))
    );
}

pub mod aquila_favonia {
    use crate::{compose_status_impls, status_impls::primitives::all::*};

    pub struct AquilaFavoniaWeapon {
        pub weapon_type: WeaponType,
        pub status_id: StatusId,
    }

    struct AquilaFavoniaEvent();
    impl OpponentCharacterSkillEvent for AquilaFavoniaEvent {
        fn invoke(e: &mut TriggerEventContext<XEvent>, _: XEventSkill) -> Option<AppliedEffectResult> {
            if !e.attached_character_is_active() {
                return None;
            }

            e.consume_counter(|e, _| {
                e.add_cmd(Command::Heal(1));
            })
        }
    }

    compose_status_impls!(AquilaFavoniaWeapon(
        IncreaseOutgoingDMG::new(1, AppliedEffectResult::NoChange),
        OpponentCharacterSkillEventI(AquilaFavoniaEvent())
    ));
    card_impl_for_weapon!(AquilaFavoniaWeapon);

    weapon!(
        AquilaFavoniaWeapon,
        "Aquila Favonia",
        Sword,
        AquilaFavonia,
        Cost::aligned(3),
        status => equipment_status(NAME).with_counter(CounterSpec::new("[Counter]", 2).resets_at_turn_end(true))
    );
}
