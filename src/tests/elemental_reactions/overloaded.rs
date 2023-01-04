use super::*;

#[test]
fn test_overloaded_force_switch() {
    let mut gs = GameState::new(
        &vector![CharId::Yoimiya, CharId::Fischl],
        &vector![CharId::Yoimiya, CharId::Fischl],
        true,
    );
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert_eq!(1, gs.players.1.active_char_index);
    assert_eq!(4, gs.players.1.char_states[0].get_hp());
    assert_eq!(10, gs.players.1.char_states[1].get_hp());
    assert!(gs.players.0.char_states[0].applied.is_empty());
}

#[test]
fn test_overloaded_force_switch_no_alternatives() {
    let mut gs = GameState::new(
        &vector![CharId::Yoimiya, CharId::Fischl],
        &vector![CharId::Yoimiya, CharId::Fischl],
        true,
    );
    gs.ignore_costs = true;
    gs.players.1.char_states[1].set_hp(0);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert_eq!(0, gs.players.1.active_char_index);
}

#[test]
fn test_overloaded_force_switch_rotate() {
    let mut gs = GameState::new(
        &vector![CharId::Yoimiya, CharId::Fischl],
        &vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
        true,
    );
    gs.ignore_costs = true;
    gs.players.1.char_states[0].set_hp(0);
    gs.players.1.active_char_index = 2;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert_eq!(1, gs.players.1.active_char_index);
}
