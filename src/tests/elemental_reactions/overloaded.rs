use super::*;

#[test]
fn overloaded_force_switch() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Fischl],
        vector![CharId::Yoimiya, CharId::Fischl],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert_eq!(1, gs.players.1.active_char_idx);
    assert_eq!(4, gs.players.1.char_states[0].get_hp());
    assert_eq!(10, gs.players.1.char_states[1].get_hp());
    assert!(gs.players.0.char_states[0].applied.is_empty());
}

#[test]
fn overloaded_force_switch_no_alternatives() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Fischl],
        vector![CharId::Yoimiya, CharId::Fischl],
    )
    .ignore_costs(true)
    .build();
    gs.players.1.char_states[1].set_hp(0);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert_eq!(0, gs.players.1.active_char_idx);
}

#[test]
fn overloaded_force_switch_rotate() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Fischl],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .ignore_costs(true)
    .build();
    gs.players.1.char_states[0].set_hp(0);
    gs.players.1.active_char_idx = 2;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert_eq!(1, gs.players.1.active_char_idx);
}
