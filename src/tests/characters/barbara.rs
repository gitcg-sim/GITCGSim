use super::*;

#[test]
fn test_melody_loop_heals_and_applies_hydro_to_active_character() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(
            vector![CharId::Barbara, CharId::Noelle],
            vector![CharId::Fischl, CharId::Yoimiya],
        )
        .with_enable_log(true)
        .build()
    };
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
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
    assert!(gs.get_player(PlayerId::PlayerFirst).has_summon(SummonId::MelodyLoop));
    assert_eq!(4, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[1].get_hp());
    gs.advance_multiple(&vec![Input::NoAction]);
    assert_eq!(5, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    assert_eq!(9, gs.get_player(PlayerId::PlayerFirst).char_states[1].get_hp());
    assert_eq!(
        elem_set![Element::Hydro],
        gs.get_player(PlayerId::PlayerFirst).char_states[0].applied
    );
}
