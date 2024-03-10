use super::*;

#[test]
fn gale_blade_forces_switch_1_character() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Jean], vector![CharId::Ganyu])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::GaleBlade),
    )]);
    assert_eq!(0, gs.player(PlayerId::PlayerSecond).active_char_idx);
    assert!(gs.player(PlayerId::PlayerSecond).active_character().applied.is_empty());
}

#[test]
fn gale_blade_forces_switch_to_prev() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Jean],
        vector![CharId::Ganyu, CharId::Yoimiya, CharId::Fischl],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::GaleBlade),
    )]);
    assert_eq!(1, gs.player(PlayerId::PlayerSecond).active_char_idx);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GaleBlade)),
    ]);
    assert_eq!(2, gs.player(PlayerId::PlayerSecond).active_char_idx);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::GaleBlade),
    )]);
    assert_eq!(0, gs.player(PlayerId::PlayerSecond).active_char_idx);
}

#[test]
fn dandelion_breeze_heals_all_and_summons_dandelion_field() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Jean, CharId::Ningguang, CharId::FatuiPyroAgent],
        vector![CharId::Ganyu, CharId::Yoimiya, CharId::Fischl],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    {
        let player = gs.player_mut(PlayerId::PlayerFirst);
        player.char_states[0].reduce_hp(5);
        player.char_states[1].reduce_hp(5);
        player.char_states[2].reduce_hp(5);
    }
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::DandelionBreeze),
    )]);
    {
        let player = gs.player(PlayerId::PlayerFirst);
        assert_eq!(7, player.char_states[0].hp());
        assert_eq!(7, player.char_states[1].hp());
        assert_eq!(7, player.char_states[2].hp());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    {
        let player = gs.player(PlayerId::PlayerFirst);
        assert_eq!(7, player.char_states[0].hp());
        assert_eq!(8, player.char_states[1].hp());
        assert_eq!(7, player.char_states[2].hp());
    }
    assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
}
