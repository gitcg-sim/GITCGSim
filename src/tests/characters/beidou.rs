use super::*;

#[test]
fn tidecaller_prepared_skill_not_discarded_by_lost_shield_points() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Beidou, CharId::Noelle], vector![CharId::Fischl])
            .enable_log(true)
            .ignore_costs(true)
            .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Tidecaller),
    )]);
    assert_eq!(Some(PlayerId::PlayerSecond), gs.to_move_player());
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(0, StatusId::TidecallerSurfEmbrace));
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .has_shield_points());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(0, StatusId::TidecallerSurfEmbrace));
    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .has_shield_points());
    assert_eq!(10, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    assert_eq!(None, gs.to_move_player());
    gs.advance_multiple([Input::NoAction]);
    assert_eq!(Some(PlayerId::PlayerSecond), gs.to_move_player());
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        elem_set![Element::Electro],
        gs.get_player(PlayerId::PlayerSecond).char_states[0].applied
    );
}

#[test]
fn tidecaller_prepared_skill_interrupted_by_overload() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Beidou, CharId::Noelle], vector![CharId::Fischl])
            .enable_log(true)
            .ignore_costs(true)
            .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Tidecaller),
    )]);
    assert_eq!(Some(PlayerId::PlayerSecond), gs.to_move_player());
    gs.get_player_mut(PlayerId::PlayerFirst).char_states[0]
        .applied
        .insert(Element::Pyro);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(0, StatusId::TidecallerSurfEmbrace));
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .has_shield_points());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::Nightrider),
    )]);
    assert_eq!(1, gs.get_player(PlayerId::PlayerFirst).active_char_idx);
    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(0, StatusId::TidecallerSurfEmbrace));
    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .has_shield_points());
    assert_eq!(9, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    assert_eq!(Some(PlayerId::PlayerFirst), gs.to_move_player());
}

// TOOD don't know the actual interaction between prepare skill and frozen
#[test]
fn tidecaller_prepared_skill_frozen() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Beidou, CharId::Noelle],
        vector![CharId::SangonomiyaKokomi],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Tidecaller),
    )]);
    assert_eq!(Some(PlayerId::PlayerSecond), gs.to_move_player());
    gs.get_player_mut(PlayerId::PlayerFirst).char_states[0]
        .applied
        .insert(Element::Cryo);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(0, StatusId::TidecallerSurfEmbrace));
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .has_shield_points());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::TheShapeOfWater),
    )]);
    assert_eq!(0, gs.get_player(PlayerId::PlayerFirst).active_char_idx);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(0, StatusId::TidecallerSurfEmbrace));
    assert_eq!(None, gs.to_move_player());
    gs.advance_multiple([Input::NoAction]);
    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(0, StatusId::TidecallerSurfEmbrace));
    assert_eq!(Some(PlayerId::PlayerFirst), gs.to_move_player());
}
