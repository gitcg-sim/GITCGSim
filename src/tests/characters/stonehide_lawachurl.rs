use super::*;

#[test]
fn stonehide_consumes_2_usages_for_geo_dmg() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::StonehideLawachurl], vector![CharId::Ningguang])
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    assert_eq!(8, gs.player(PlayerId::PlayerFirst).char_states[0].hp());
    assert_eq!(
        3,
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Character(0, StatusId::Stonehide))
            .unwrap()
            .usages()
    );
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::SparklingScatter),
        ),
    ]);
    assert_eq!(
        1,
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Character(0, StatusId::Stonehide))
            .unwrap()
            .usages()
    );
    assert_eq!(8, gs.player(PlayerId::PlayerFirst).char_states[0].hp());
}

#[test]
fn stonehide_removes_stone_force_at_zero_usages() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::StonehideLawachurl], vector![CharId::Ningguang])
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    assert_eq!(8, gs.player(PlayerId::PlayerFirst).char_states[0].hp());
    assert_eq!(
        3,
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Character(0, StatusId::Stonehide))
            .unwrap()
            .usages()
    );
    gs.advance_multiple([
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
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Character(0, StatusId::Stonehide))
    );
    assert_eq!(
        None,
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Character(0, StatusId::StoneForce))
    );
}

#[test]
fn stone_force_infuses_geo() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::StonehideLawachurl], vector![CharId::Ningguang])
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    assert_eq!(8, gs.player(PlayerId::PlayerFirst).char_states[0].hp());
    assert!(gs
        .status_collection_mut(PlayerId::PlayerFirst)
        .get(StatusKey::Character(0, StatusId::StoneForce))
        .is_some());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::PlamaLawa),
    )]);
    assert!(gs
        .status_collection_mut(PlayerId::PlayerFirst)
        .get(StatusKey::Team(StatusId::CrystallizeShield))
        .is_some());
    assert_eq!(6, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::PlamaLawa)),
    ]);
    assert_eq!(4, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
}
