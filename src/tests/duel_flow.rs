use enumset::enum_set;

use super::*;

#[test]
fn test_first_phase_is_roll_phase() {
    let gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Kaeya, CharId::Fischl], vector![CharId::KamisatoAyaka])
        .with_enable_log(true)
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
fn test_action_phase_and_first_player_to_end_round() {
    let mut gs =
        GameStateBuilder::new_roll_phase_1(vector![CharId::Kaeya, CharId::Fischl], vector![CharId::KamisatoAyaka])
            .with_enable_log(true)
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

    gs.advance_multiple(&vec![NO_ACTION]);
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
fn test_post_death_switch() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Fischl],
        vector![CharId::Kaeya, CharId::Ganyu, CharId::Yoimiya],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    {
        let p1 = &mut gs.players.1;
        p1.char_states[0].set_hp(1);
        p1.char_states[1].set_hp(1);
        p1.char_states[2].set_hp(1);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(2)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::NiwabiFireDance),
        ),
    ]);
}

#[test]
fn test_piercing_dmg_victory() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Ganyu],
        vector![CharId::Kaeya, CharId::Ganyu, CharId::Yoimiya],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    {
        let p1 = &mut gs.players.1;
        p1.char_states[0].set_hp(1);
        p1.char_states[1].set_hp(1);
        p1.char_states[2].set_hp(1);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
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
fn test_piercing_dmg_causing_post_death_switch() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Ganyu],
        vector![CharId::Kaeya, CharId::Ganyu, CharId::Yoimiya],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    {
        let p1 = &mut gs.players.1;
        p1.char_states[0].set_hp(1);
        p1.char_states[1].set_hp(1);
        p1.char_states[2].set_hp(8);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FrostflakeArrow)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(2)),
    ]);
}

#[test]
fn test_trigger_effects_after_post_death_switch() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Fischl], vector![CharId::Kaeya, CharId::Yoimiya])
        .with_enable_log(true)
        .build();
    gs.ignore_costs = true;
    {
        let p1 = &mut gs.players.1;
        p1.char_states[0].set_hp(4);
        p1.char_states[1].set_hp(1);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::GlacialWaltz)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(1)),
    ]);
    assert!(gs.players.1.has_team_status(StatusId::Icicle));
    assert_eq!(7, gs.players.0.char_states[0].get_hp());
}

#[test]
fn test_end_phase_post_death_switch() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Yoimiya, CharId::Ganyu],
        vector![CharId::Fischl, CharId::KamisatoAyaka, CharId::Collei],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    {
        let yoimiya = &mut gs.players.1.char_states[0];
        yoimiya.set_hp(6);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
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
fn test_end_phase_winner_decided() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Yoimiya],
        vector![CharId::Fischl, CharId::KamisatoAyaka, CharId::Collei],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    {
        let yoimiya = &mut gs.players.1.char_states[0];
        yoimiya.set_hp(6);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
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
fn test_play_card() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Yoimiya],
        vector![CharId::Fischl, CharId::KamisatoAyaka, CharId::Collei],
    )
    .with_enable_log(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand = vector![CardId::BlankCard, CardId::Starsigns, CardId::TheBestestTravelCompanion];
    gs.players.0.dice[Dice::PYRO] = 1;
    gs.players.0.dice[Dice::DENDRO] = 1;
    assert_eq!(0, gs.players.0.dice.omni);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::TheBestestTravelCompanion, None),
    )]);
    assert_eq!(2, gs.players.0.dice.omni);
    assert_eq!(0, gs.players.0.get_active_character().get_energy());
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::Starsigns, None),
    )]);
    assert_eq!(0, gs.players.0.dice.omni);
    assert_eq!(1, gs.players.0.get_active_character().get_energy());
}

#[test]
fn test_elemental_tuning() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Yoimiya], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.dice[Dice::DENDRO] = 1;
    gs.players.0.hand = vector![
        CardId::BlankCard,
        CardId::Starsigns,
        CardId::TheBestestTravelCompanion,
        CardId::BlankCard
    ];
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::ElementalTuning(CardId::TheBestestTravelCompanion),
    )]);
    assert_eq!(0, gs.players.0.dice[Dice::DENDRO]);
    assert_eq!(1, gs.players.0.dice[Dice::PYRO]);
}

#[test]
fn test_artifact_equip_replace() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Yoimiya], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand = vector![CardId::WitchsScorchingHat, CardId::BrokenRimesEcho];
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::BrokenRimesEcho, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        StatusKey::Equipment(0, EquipSlot::Artifact, StatusId::BrokenRimesEcho),
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .find_equipment(0, EquipSlot::Artifact)
            .unwrap()
            .key
    );

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::WitchsScorchingHat, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        StatusKey::Equipment(0, EquipSlot::Artifact, StatusId::WitchsScorchingHat),
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .find_equipment(0, EquipSlot::Artifact)
            .unwrap()
            .key
    );
}

#[test]
fn test_weapon_equip_replace() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Yoimiya], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand = vector![CardId::SkywardHarp, CardId::SacrificialBow];
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::SacrificialBow, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        StatusKey::Equipment(0, EquipSlot::Weapon, StatusId::SacrificialBow),
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .find_equipment(0, EquipSlot::Weapon)
            .unwrap()
            .key
    );

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::SkywardHarp, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        StatusKey::Equipment(0, EquipSlot::Weapon, StatusId::SkywardHarp),
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .find_equipment(0, EquipSlot::Weapon)
            .unwrap()
            .key
    );
}

#[test]
fn test_skill_cast_tracker() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Ganyu, CharId::Yoimiya],
        vector![CharId::Fischl, CharId::Noelle],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
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

    gs.advance_multiple(&vec![
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
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
    ]);
    {
        let yoimiya = gs.get_player(PlayerId::PlayerFirst).char_states[1];
        assert_eq!(enum_set![CharFlag::SkillCastedThisTurn1], yoimiya.flags);
    }
    gs.advance_multiple(&vec![
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
