use super::*;

#[test]
fn test_icetide_vortex() {
    let mut gs = GameState::new(&vector![CharId::Eula], &vector![CharId::Yoimiya], true);
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::IcetideVortex),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Character(0, StatusId::Grimheart))
        .is_some());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::IcetideVortex)),
    ]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Character(0, StatusId::Grimheart))
        .is_none());
}

#[test]
fn test_glacial_illumination_prevents_energy_gain_and_increments_counter_and_deals_physical_dmg() {
    let mut gs = GameState::new(
        &vector![CharId::Eula],
        &vector![CharId::Xiangling, CharId::Fischl, CharId::Kaeya],
        true,
    );
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::GlacialIllumination),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Summon(SummonId::LightfallSword))
        .is_some());
    assert_eq!(
        0,
        gs.get_player(PlayerId::PlayerFirst).get_active_character().get_energy()
    );
    let get_counter = |gs: &GameState| {
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Summon(SummonId::LightfallSword))
            .unwrap()
            .get_counter()
    };
    assert_eq!(0, get_counter(&gs));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::IcetideVortex)),
    ]);
    {
        let fischl = &gs.get_player(PlayerId::PlayerSecond).char_states[1];
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, fischl.get_hp());
    }
    assert_eq!(
        0,
        gs.get_player(PlayerId::PlayerFirst).get_active_character().get_energy()
    );
    assert_eq!(2, get_counter(&gs));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkEdel),
        ),
    ]);
    {
        let kaeya = &gs.get_player(PlayerId::PlayerSecond).char_states[2];
        assert_eq!(elem_set![], kaeya.applied);
        assert_eq!(8, kaeya.get_hp());
    }
    assert_eq!(
        0,
        gs.get_player(PlayerId::PlayerFirst).get_active_character().get_energy()
    );
    assert_eq!(4, get_counter(&gs));

    // Set HP to 10 and Unapply Cryo
    {
        let xiangling = &mut gs.get_player_mut(PlayerId::PlayerSecond).char_states[0];
        xiangling.set_hp(10);
        xiangling.applied = elem_set![];
    }

    // Run burst
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    {
        let xiangling = &gs.get_player(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(elem_set![], xiangling.applied);
        assert_eq!(4, xiangling.get_hp());
    }
}

#[test]
fn test_glacial_illumination_does_not_accumulate_counter_on_others() {
    let mut gs = GameState::new(
        &vector![CharId::Eula, CharId::Fischl],
        &vector![CharId::Xiangling],
        true,
    );
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::GlacialIllumination),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Summon(SummonId::LightfallSword))
        .is_some());
    let get_counter = |gs: &GameState| {
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Summon(SummonId::LightfallSword))
            .unwrap()
            .get_counter()
    };
    assert_eq!(0, get_counter(&gs));
}
