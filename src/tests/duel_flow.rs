use super::*;

#[test]
fn first_phase_is_roll_phase() {
    let gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Kaeya, CharId::Fischl],
        vector![CharId::KamisatoAyaka],
    )
    .build();
    assert_eq!(1, gs.round_number);
    assert_eq!(
        Phase::RollPhase {
            first_active_player: PlayerId::PlayerFirst,
            roll_phase_state: RollPhaseState::Start
        },
        gs.phase
    );
}

#[test]
fn action_phase_and_first_player_to_end_round() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Kaeya, CharId::Fischl],
        vector![CharId::KamisatoAyaka],
    )
    .build();
    gs.advance_roll_phase_no_dice();
    assert_eq!(
        Phase::ActionPhase {
            first_end_round: None,
            active_player: PlayerId::PlayerFirst
        },
        gs.phase
    );
    gs.advance(Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound))
        .unwrap();
    assert_eq!(
        Phase::ActionPhase {
            first_end_round: Some(PlayerId::PlayerFirst),
            active_player: PlayerId::PlayerSecond
        },
        gs.phase
    );
    gs.advance(Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound))
        .unwrap();
    assert_eq!(
        Phase::EndPhase {
            next_first_active_player: PlayerId::PlayerFirst
        },
        gs.phase
    );

    gs.advance_multiple([NO_ACTION]);
    assert_eq!(2, gs.round_number);
    assert_eq!(
        Phase::RollPhase {
            first_active_player: PlayerId::PlayerFirst,
            roll_phase_state: RollPhaseState::Start
        },
        gs.phase
    );

    gs.advance_roll_phase_no_dice();
    assert_eq!(
        Phase::ActionPhase {
            first_end_round: None,
            active_player: PlayerId::PlayerFirst
        },
        gs.phase
    );
}

#[test]
fn post_death_switch() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Fischl],
        vector![CharId::Kaeya, CharId::Ganyu, CharId::Yoimiya],
    )
    .ignore_costs(true)
    .build();
    {
        let p1 = &mut gs.players.1;
        p1.char_states[0].set_hp(1);
        p1.char_states[1].set_hp(1);
        p1.char_states[2].set_hp(1);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(2)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::NiwabiFireDance),
        ),
    ]);
}

#[test]
fn piercing_dmg_victory() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Ganyu],
        vector![CharId::Kaeya, CharId::Ganyu, CharId::Yoimiya],
    )
    .ignore_costs(true)
    .build();
    {
        let p1 = &mut gs.players.1;
        p1.char_states[0].set_hp(1);
        p1.char_states[1].set_hp(1);
        p1.char_states[2].set_hp(1);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FrostflakeArrow),
    )]);
    assert_eq!(
        Phase::WinnerDecided {
            winner: PlayerId::PlayerFirst
        },
        gs.phase
    );
}

#[test]
fn piercing_dmg_causing_post_death_switch() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Ganyu],
        vector![CharId::Kaeya, CharId::Ganyu, CharId::Yoimiya],
    )
    .ignore_costs(true)
    .build();
    {
        let p1 = &mut gs.players.1;
        p1.char_states[0].set_hp(1);
        p1.char_states[1].set_hp(1);
        p1.char_states[2].set_hp(8);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FrostflakeArrow)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(2)),
    ]);
}

#[test]
fn trigger_effects_after_post_death_switch() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Fischl], vector![CharId::Kaeya, CharId::Yoimiya])
            .ignore_costs(true)
            .build();
    {
        let p1 = &mut gs.players.1;
        p1.char_states[0].set_hp(4);
        p1.char_states[1].set_hp(1);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::GlacialWaltz)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(1)),
    ]);
    assert!(gs.has_team_status(PlayerId::PlayerSecond, StatusId::Icicle));
    assert_eq!(7, gs.players.0.char_states[0].get_hp());
}

#[test]
fn end_phase_post_death_switch() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Ganyu],
        vector![CharId::Fischl, CharId::KamisatoAyaka, CharId::Collei],
    )
    .ignore_costs(true)
    .build();
    {
        let yoimiya = &mut gs.players.1.char_states[0];
        yoimiya.set_hp(6);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::KamisatoArtSoumetsu),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::TrumpCardKitty)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        NO_ACTION,
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PostDeathSwitch(1)),
        NO_ACTION,
    ]);
    // No crashes
}

#[test]
fn end_phase_winner_decided() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya],
        vector![CharId::Fischl, CharId::KamisatoAyaka, CharId::Collei],
    )
    .ignore_costs(true)
    .build();
    {
        let yoimiya = &mut gs.players.1.char_states[0];
        yoimiya.set_hp(6);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::KamisatoArtSoumetsu),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::TrumpCardKitty)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        NO_ACTION,
    ]);
    assert_eq!(
        Phase::WinnerDecided {
            winner: PlayerId::PlayerSecond
        },
        gs.phase
    );
}

#[test]
fn play_card() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya],
        vector![CharId::Fischl, CharId::KamisatoAyaka, CharId::Collei],
    )
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand = [CardId::BlankCard, CardId::Starsigns, CardId::TheBestestTravelCompanion].into();
    gs.players.0.dice[Dice::PYRO] = 1;
    gs.players.0.dice[Dice::DENDRO] = 1;
    assert_eq!(0, gs.players.0.dice.omni);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::TheBestestTravelCompanion, None),
    )]);
    assert_eq!(2, gs.players.0.dice.omni);
    assert_eq!(0, gs.players.0.get_active_character().get_energy());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::Starsigns, None),
    )]);
    assert_eq!(0, gs.players.0.dice.omni);
    assert_eq!(1, gs.players.0.get_active_character().get_energy());
}

#[test]
fn elemental_tuning() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya], vector![CharId::Fischl]).build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.dice[Dice::DENDRO] = 1;
    gs.players.0.hand = [
        CardId::BlankCard,
        CardId::Starsigns,
        CardId::TheBestestTravelCompanion,
        CardId::BlankCard,
    ]
    .into();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::ElementalTuning(CardId::TheBestestTravelCompanion),
    )]);
    assert_eq!(0, gs.players.0.dice[Dice::DENDRO]);
    assert_eq!(1, gs.players.0.dice[Dice::PYRO]);
}

#[test]
fn artifact_equip_replace() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya], vector![CharId::Fischl])
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand = [CardId::WitchsScorchingHat, CardId::BrokenRimesEcho].into();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::BrokenRimesEcho, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        StatusKey::Equipment(0, EquipSlot::Artifact, StatusId::BrokenRimesEcho),
        gs.get_status_collection_mut(PlayerId::PlayerFirst)
            .find_equipment(0, EquipSlot::Artifact)
            .unwrap()
            .key
    );

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::WitchsScorchingHat, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        StatusKey::Equipment(0, EquipSlot::Artifact, StatusId::WitchsScorchingHat),
        gs.get_status_collection_mut(PlayerId::PlayerFirst)
            .find_equipment(0, EquipSlot::Artifact)
            .unwrap()
            .key
    );
}

#[test]
fn weapon_equip_replace() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya], vector![CharId::Fischl])
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand = [CardId::SkywardHarp, CardId::SacrificialBow].into();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::SacrificialBow, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        StatusKey::Equipment(0, EquipSlot::Weapon, StatusId::SacrificialBow),
        gs.get_status_collection_mut(PlayerId::PlayerFirst)
            .find_equipment(0, EquipSlot::Weapon)
            .unwrap()
            .key
    );

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::SkywardHarp, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        StatusKey::Equipment(0, EquipSlot::Weapon, StatusId::SkywardHarp),
        gs.get_status_collection_mut(PlayerId::PlayerFirst)
            .find_equipment(0, EquipSlot::Weapon)
            .unwrap()
            .key
    );
}

#[test]
fn skill_cast_tracker() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Ganyu, CharId::Yoimiya],
        vector![CharId::Fischl, CharId::Noelle],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FrostflakeArrow)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
    ]);
    {
        let ganyu = gs.get_player(PlayerId::PlayerFirst).get_active_character();
        assert_eq!(enum_set![CharFlag::SkillCastedThisTurn2], ganyu.flags);
    }
    {
        let yoimiya = gs.get_player(PlayerId::PlayerFirst).char_states[1];
        assert_eq!(enum_set![], yoimiya.flags);
    }
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(enum_set![CharFlag::SkillCastedThisTurn0], fischl.flags);
    }
    {
        let noelle = gs.get_player(PlayerId::PlayerSecond).char_states[1];
        assert_eq!(enum_set![], noelle.flags);
    }

    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FrostflakeArrow)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    {
        let ganyu = gs.get_player(PlayerId::PlayerFirst).get_active_character();
        assert_eq!(enum_set![CharFlag::SkillCastedThisTurn2], ganyu.flags);
    }
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(
            enum_set![CharFlag::SkillCastedThisTurn0 | CharFlag::SkillCastedThisTurn1],
            fischl.flags
        );
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
    ]);
    {
        let yoimiya = gs.get_player(PlayerId::PlayerFirst).char_states[1];
        assert_eq!(enum_set![CharFlag::SkillCastedThisTurn1], yoimiya.flags);
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
        Input::NoAction,
    ]);
    assert_eq!(2, gs.round_number);
    {
        let ganyu = gs.get_player(PlayerId::PlayerFirst).get_active_character();
        assert_eq!(enum_set![], ganyu.flags);
    }
    {
        let yoimiya = gs.get_player(PlayerId::PlayerFirst).char_states[1];
        assert_eq!(enum_set![], yoimiya.flags);
    }
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(enum_set![], fischl.flags);
    }
    {
        let noelle = gs.get_player(PlayerId::PlayerSecond).char_states[1];
        assert_eq!(enum_set![], noelle.flags);
    }
}

#[test]
fn test_select_starting_plunging_attack_flags() {
    let mut gs = GameStateInitializer::new(vector![CharId::Yoimiya], vector![CharId::Kaeya, CharId::Yoimiya])
        .ignore_costs(true)
        .start_at_select_character()
        .build();
    gs.advance(Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::SwitchCharacter(0),
    ))
    .unwrap();
    gs.advance(Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::SwitchCharacter(1),
    ))
    .unwrap();
    assert_eq!(0, gs.get_player(PlayerId::PlayerFirst).active_char_idx);
    assert_eq!(1, gs.get_player(PlayerId::PlayerSecond).active_char_idx);
    assert_eq!(enum_set![], gs.get_player(PlayerId::PlayerFirst).char_states[0].flags);
    assert_eq!(enum_set![], gs.get_player(PlayerId::PlayerSecond).char_states[0].flags);
    assert_eq!(enum_set![], gs.get_player(PlayerId::PlayerSecond).char_states[1].flags);
    assert_eq!(vec![Input::NoAction], gs.available_actions().to_vec());
    assert_eq!(Phase::new_roll_phase(PlayerId::PlayerFirst), gs.phase);
    gs.advance_roll_phase_no_dice();
    assert!(gs.get_player(PlayerId::PlayerFirst).char_states[0]
        .flags
        .contains(CharFlag::PlungingAttack));
    assert!(!gs.get_player(PlayerId::PlayerSecond).char_states[0]
        .flags
        .contains(CharFlag::PlungingAttack));
    assert!(gs.get_player(PlayerId::PlayerSecond).char_states[1]
        .flags
        .contains(CharFlag::PlungingAttack));
}

#[test]
fn hand_size_limit() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya], vector![CharId::Fischl]).build();
    gs.advance_roll_phase_no_dice();
    let mut hand = [CardId::BlankCard; PlayerState::HAND_SIZE_LIMIT];
    hand[0] = CardId::Strategize;
    gs.players.0.hand = hand.into();
    gs.players.0.dice[Dice::Omni] = 1;
    assert_eq!(PlayerState::HAND_SIZE_LIMIT, gs.players.0.hand_len() as usize);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Strategize, None)),
        Input::NondetResult(NondetResult::ProvideCards(
            (list8![CardId::BlankCard, CardId::BlankCard], Default::default()).into(),
        )),
    ]);
    assert_eq!(PlayerState::HAND_SIZE_LIMIT, gs.players.0.hand_len() as usize);
}

#[test]
fn auto_cost_payment_for_switching_based_on_switch_target() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya, CharId::Ganyu], vector![CharId::Fischl])
            .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.dice[Dice::PYRO] = 1;
    gs.players.0.dice[Dice::CRYO] = 1;
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::SwitchCharacter(1),
    )]);
    assert_eq!(DiceCounter::elem(Element::Cryo, 1), gs.players.0.dice);
}
