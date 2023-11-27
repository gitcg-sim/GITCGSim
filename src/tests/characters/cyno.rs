use super::*;

#[test]
fn indwelling_level_increase_at_end_phase() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Cyno], vector![CharId::Yoimiya])
        .enable_log(true)
        .build();
    macro_rules! assert_counter {
        ($n: expr) => {
            assert_eq!(
                $n,
                gs.get_player(PlayerId::PlayerFirst)
                    .status_collection
                    .get(StatusKey::Character(0, StatusId::PactswornPathclearer))
                    .unwrap()
                    .get_counter()
            );
        };
    }
    gs.ignore_costs = true;

    for i in 0..=5 {
        gs.advance_roll_phase_no_dice();
        assert_counter!(i);
        gs.advance_multiple(&vec![
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
            Input::NoAction,
        ]);
    }

    gs.advance_roll_phase_no_dice();
    assert_counter!(6 - 4);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert_counter!(6 - 4 + 1);
}

#[test]
fn indwelling_level_ge_2_electro_infusion() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Cyno], vector![CharId::Yoimiya])
        .enable_log(true)
        .ignore_costs(true)
        .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::InvokersSpear)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
    assert_eq!(
        elem_set![],
        gs.get_player(PlayerId::PlayerSecond).get_active_character().applied
    );

    for level in 2..=5 {
        let mut gs = gs.clone();
        gs.get_player_mut(PlayerId::PlayerFirst)
            .status_collection
            .get_mut(StatusKey::Character(0, StatusId::PactswornPathclearer))
            .unwrap()
            .set_counter(level);
        gs.advance_multiple(&vec![Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::InvokersSpear),
        )]);
        assert!(gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp() <= 6);
        assert_eq!(
            elem_set![Element::Electro],
            gs.get_player(PlayerId::PlayerSecond).get_active_character().applied
        );
    }
}

#[test]
fn indwelling_level_ge_4_increases_dmg_by_2() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Cyno], vector![CharId::Yoimiya])
        .enable_log(true)
        .ignore_costs(true)
        .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::InvokersSpear)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);

    for level in 4..=5 {
        gs.get_player_mut(PlayerId::PlayerFirst)
            .status_collection
            .get_mut(StatusKey::Character(0, StatusId::PactswornPathclearer))
            .unwrap()
            .set_counter(level);

        {
            let mut gs = gs.clone();
            gs.advance_multiple(&vec![Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::CastSkill(SkillId::InvokersSpear),
            )]);
            assert_eq!(4, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
        }
        {
            let mut gs = gs.clone();
            gs.advance_multiple(&vec![Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::CastSkill(SkillId::SecretRiteChasmicSoulfarer),
            )]);
            assert_eq!(3, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
        }
    }
}

#[test]
fn sacred_rite_wolfs_swiftness_uses_indwelling_level_pre_increase_and_increases_indwelling_level_by_2() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Cyno], vector![CharId::Yoimiya])
        .enable_log(true)
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();

    for level in 0..=5 {
        let mut gs = gs.clone();
        gs.get_player_mut(PlayerId::PlayerFirst)
            .status_collection
            .get_mut(StatusKey::Character(0, StatusId::PactswornPathclearer))
            .unwrap()
            .set_counter(level);

        gs.advance_multiple(&vec![Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::SacredRiteWolfsSwiftness),
        )]);
        assert_eq!(
            10 - 4 - (if level >= 4 { 2 } else { 0 }),
            gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp()
        );
        assert_eq!(
            level + 2 - (if level + 2 >= 6 { 4 } else { 0 }),
            gs.get_player(PlayerId::PlayerFirst)
                .status_collection
                .get(StatusKey::Character(0, StatusId::PactswornPathclearer))
                .unwrap()
                .get_counter()
        );
    }
}
