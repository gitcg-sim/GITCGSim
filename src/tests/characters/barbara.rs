use super::*;

#[test]
fn melody_loop_heals_and_applies_hydro_to_active_character() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Barbara, CharId::Noelle],
        vector![CharId::Fischl, CharId::Yoimiya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::LetTheShowBegin)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::MelodyLoop));
    assert_eq!(4, gs.player(PlayerId::PlayerFirst).char_states[0].hp());
    assert_eq!(8, gs.player(PlayerId::PlayerFirst).char_states[1].hp());
    gs.advance_multiple([Input::NoAction]);
    assert_eq!(5, gs.player(PlayerId::PlayerFirst).char_states[0].hp());
    assert_eq!(9, gs.player(PlayerId::PlayerFirst).char_states[1].hp());
    assert_eq!(
        elem_set![Element::Hydro],
        gs.player(PlayerId::PlayerFirst).char_states[0].applied
    );
}
