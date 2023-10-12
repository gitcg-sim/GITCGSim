use super::*;

#[test]
fn test_stonehide_consumes_2_usages_for_geo_dmg() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::StonehideLawachurl], vector![CharId::Ningguang])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    assert_eq!(
        3,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Character(0, StatusId::Stonehide))
            .unwrap()
            .get_usages()
    );
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::SparklingScatter),
        ),
    ]);
    assert_eq!(
        1,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Character(0, StatusId::Stonehide))
            .unwrap()
            .get_usages()
    );
    assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
}

#[test]
fn test_stonehide_removes_stone_force_at_zero_usages() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::StonehideLawachurl], vector![CharId::Ningguang])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    assert_eq!(
        3,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Character(0, StatusId::Stonehide))
            .unwrap()
            .get_usages()
    );
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::SparklingScatter),
        ),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::SparklingScatter),
        ),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::SparklingScatter),
        ),
    ]);
    assert_eq!(
        None,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Character(0, StatusId::Stonehide))
    );
    assert_eq!(
        None,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Character(0, StatusId::StoneForce))
    );
}

#[test]
fn test_stone_force_infuses_geo() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::StonehideLawachurl], vector![CharId::Ningguang])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Character(0, StatusId::StoneForce))
        .is_some());
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::PlamaLawa),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Team(StatusId::CrystallizeShield))
        .is_some());
    assert_eq!(6, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::PlamaLawa)),
    ]);
    assert_eq!(4, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
}
