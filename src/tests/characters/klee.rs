use super::*;

#[test]
fn charged_attack_affected_by_explosive_spark() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Klee], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.dice.add_in_place(&DiceCounter::omni(9));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::JumpyDumpty)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(6, gs.players.0.dice.total());
    assert_eq!(7, gs.players.1.char_states[0].get_hp());
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Kaboom),
    )]);
    // Charged Attack
    assert_eq!(4, gs.players.0.dice.total());
    assert_eq!(5, gs.players.1.char_states[0].get_hp());
    assert!(!gs.players.0.has_active_character_status(StatusId::ExplosiveSpark));
}

#[test]
fn normal_attack_not_affected_by_explosive_spark() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Klee], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.dice.add_in_place(&DiceCounter::omni(8));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::JumpyDumpty)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(
        1,
        gs.players
            .0
            .status_collection
            .get(StatusKey::Character(0, StatusId::ExplosiveSpark))
            .unwrap()
            .get_usages()
    );
    assert_eq!(5, gs.players.0.dice.total());
    assert_eq!(7, gs.players.1.char_states[0].get_hp());
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Kaboom),
    )]);
    assert_eq!(2, gs.players.0.dice.total());
    assert_eq!(6, gs.players.1.char_states[0].get_hp());
    assert!(gs.players.0.has_active_character_status(StatusId::ExplosiveSpark));
}

#[test]
fn talent_card_increases_explosive_spark_usages() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Klee], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::PoundingSurprise);
    gs.players.0.dice.add_in_place(&DiceCounter::omni(8));
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::PoundingSurprise, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        2,
        gs.players
            .0
            .status_collection
            .get(StatusKey::Character(0, StatusId::ExplosiveSpark))
            .unwrap()
            .get_usages()
    );
}

#[test]
fn klee_take_damage() {
    let mut gs =
        GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Klee], vector![CharId::Fischl, CharId::Kaeya])
            .with_enable_log(true)
            .with_ignore_costs(true)
            .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SparksNSplash),
    )]);
    assert_eq!(7, gs.players.1.char_states[0].get_hp());
    assert!(gs.players.1.has_team_status(StatusId::SparksNSplash));
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
    )]);
    assert_eq!(5, gs.players.1.char_states[0].get_hp());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::CeremonialBladework),
        ),
    ]);
    assert_eq!(8, gs.players.1.char_states[1].get_hp());
}
