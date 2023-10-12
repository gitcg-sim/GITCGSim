use super::*;

#[test]
fn test_gale_blade_forces_switch_1_character() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Jean], vector![CharId::Ganyu])
        .with_enable_log(true)
        .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::GaleBlade),
    )]);
    assert_eq!(0, gs.get_player(PlayerId::PlayerSecond).active_char_index);
    assert!(gs
        .get_player(PlayerId::PlayerSecond)
        .get_active_character()
        .applied
        .is_empty());
}

#[test]
fn test_gale_blade_forces_switch_to_prev() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Jean],
        vector![CharId::Ganyu, CharId::Yoimiya, CharId::Fischl],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::GaleBlade),
    )]);
    assert_eq!(1, gs.get_player(PlayerId::PlayerSecond).active_char_index);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GaleBlade)),
    ]);
    assert_eq!(2, gs.get_player(PlayerId::PlayerSecond).active_char_index);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::GaleBlade),
    )]);
    assert_eq!(0, gs.get_player(PlayerId::PlayerSecond).active_char_index);
}

#[test]
fn test_dandelion_breeze_heals_all_and_summons_dandelion_field() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Jean, CharId::Ningguang, CharId::FatuiPyroAgent],
        vector![CharId::Ganyu, CharId::Yoimiya, CharId::Fischl],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    {
        let player = gs.get_player_mut(PlayerId::PlayerFirst);
        player.char_states[0].reduce_hp(5);
        player.char_states[1].reduce_hp(5);
        player.char_states[2].reduce_hp(5);
    }
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::DandelionBreeze),
    )]);
    {
        let player = gs.get_player(PlayerId::PlayerFirst);
        assert_eq!(7, player.char_states[0].get_hp());
        assert_eq!(7, player.char_states[1].get_hp());
        assert_eq!(7, player.char_states[2].get_hp());
    }
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    {
        let player = gs.get_player(PlayerId::PlayerFirst);
        assert_eq!(7, player.char_states[0].get_hp());
        assert_eq!(8, player.char_states[1].get_hp());
        assert_eq!(7, player.char_states[2].get_hp());
    }
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
}
