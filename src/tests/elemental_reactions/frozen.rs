use super::*;

#[test]
fn frozen_cannot_perform_action() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Xingqiu, CharId::KamisatoAyaka, CharId::Yoimiya],
        vector![CharId::Yoimiya, CharId::Fischl],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FatalRainscreen)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtHyouka),
        ),
    ]);
    assert_eq!(
        StatusId::Frozen,
        gs.get_status_collection(PlayerId::PlayerSecond)
            .character_statuses_vec(0)[0]
            .status_id()
            .unwrap()
    );

    // switch character and end round only
    assert_eq!(2, gs.available_actions().len());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(2)),
    ]);
    assert_eq!(Some(PlayerId::PlayerSecond), gs.to_move_player());
    assert_eq!(5, gs.available_actions().len());
}

#[test]
fn frozen_broken_by_pyro() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Xingqiu, CharId::KamisatoAyaka, CharId::Yoimiya],
        vector![CharId::Yoimiya],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FatalRainscreen)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtHyouka),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    // Reset health to prevent death
    gs.players.1.char_states[0].set_hp(10);
    assert!(gs.players.1.has_active_character_status(StatusId::Frozen));
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    assert_eq!(5, gs.players.1.char_states[0].get_hp());
    // Frozen must be un-applied
    assert!(gs
        .get_status_collection(PlayerId::PlayerSecond)
        .character_statuses_vec(0)
        .is_empty());
    // Pyro is still applied after unfreezing
    assert_eq!(enum_set![Element::Pyro], gs.players.1.char_states[0].applied);
}

#[test]
fn frozen_broken_by_physical() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Xingqiu, CharId::KamisatoAyaka, CharId::Yoimiya],
        vector![CharId::Yoimiya],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FatalRainscreen)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtHyouka),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    // Reset health to prevent death
    gs.players.1.char_states[0].set_hp(10);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    assert_eq!(6, gs.players.1.char_states[0].get_hp());
    // Frozen must be un-applied
    assert!(gs
        .get_status_collection(PlayerId::PlayerSecond)
        .character_statuses_vec(0)
        .is_empty());
    // Pyro is still applied after unfreezing
    assert!(gs.players.1.char_states[0].applied.is_empty());
}

#[test]
fn frozen_unapplied_end_of_turn() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Xingqiu, CharId::KamisatoAyaka, CharId::Yoimiya],
        vector![CharId::Yoimiya],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FatalRainscreen)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtHyouka),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
    ]);
    assert!(matches![gs.phase, Phase::EndPhase { .. }]);
    assert!(gs.players.1.has_active_character_status(StatusId::Frozen));
    gs.advance_multiple([Input::NoAction]);
    assert_eq!(2, gs.round_number);
    assert!(!gs.players.1.has_active_character_status(StatusId::Frozen));
}

#[test]
fn frozen_unapplied_end_of_turn_non_active_character() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Xingqiu, CharId::KamisatoAyaka, CharId::Yoimiya],
        vector![CharId::Yoimiya, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FatalRainscreen)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtHyouka),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(matches![gs.phase, Phase::EndPhase { .. }]);
    assert!(gs.players.1.has_character_status(0, StatusId::Frozen));
    gs.advance_multiple([Input::NoAction]);
    assert_eq!(2, gs.round_number);
    assert!(!gs.players.1.has_character_status(0, StatusId::Frozen));
}

// TODO test self-freeze not taking DMG
