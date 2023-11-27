use super::*;

#[test]
fn searing_onslaught_increases_dmg_every_3rd_use_per_round() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Diluc], vector![CharId::Kaeya])
        .enable_log(true)
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    for r in 2..=3 {
        gs.advance_multiple(&vec![
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TemperedSword)),
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        ]);
        {
            let kaeya = &mut gs.get_player_mut(PlayerId::PlayerSecond).char_states[0];
            kaeya.set_hp(10);
        }
        for i in 1..=8 {
            gs.advance_multiple(&vec![Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::CastSkill(SkillId::SearingOnslaught),
            )]);
            let counter = gs
                .get_player(PlayerId::PlayerFirst)
                .status_collection
                .get(StatusKey::Character(0, StatusId::SearingOnslaughtCounter))
                .unwrap()
                .get_counter();
            assert_eq!(std::cmp::min(3, i), counter);
            let kaeya = &mut gs.get_player_mut(PlayerId::PlayerSecond).char_states[0];
            assert_eq!(if i == 3 { 5 } else { 7 }, kaeya.get_hp());
            kaeya.set_hp(10);
        }
        gs.advance_multiple(&vec![
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
            Input::NoAction,
        ]);
        gs.advance_roll_phase_no_dice();
        assert_eq!(r, gs.round_number);
        gs.advance_multiple(&vec![Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::CeremonialBladework),
        )]);
    }
}

#[test]
fn dawn_grants_pyro_infusion() {
    let mut gs =
        GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Diluc], vector![CharId::Kaeya, CharId::Fischl])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Dawn)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
    ]);
    assert_eq!(2, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::TemperedSword),
    )]);
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
    assert_eq!(
        elem_set![Element::Pyro],
        gs.get_player(PlayerId::PlayerSecond).char_states[1].applied
    );
}
