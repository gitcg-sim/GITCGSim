use super::*;

#[test]
fn secret_art_musou_shinsetsu_increases_energy() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::RaidenShogun, CharId::Noelle, CharId::Fischl],
        vector![CharId::Ganyu, CharId::Xiangling, CharId::Xingqiu],
    )
    .with_enable_log(true)
    .with_ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SecretArtMusouShinsetsu),
    )]);
    assert_eq!(
        vec![0, 2, 2],
        gs.get_player(PlayerId::PlayerFirst)
            .char_states
            .iter_valid()
            .map(CharState::get_energy)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec![0, 0, 0],
        gs.get_player(PlayerId::PlayerSecond)
            .char_states
            .iter_valid()
            .map(CharState::get_energy)
            .collect::<Vec<_>>()
    );
}

#[test]
fn eye_of_stormy_judgment_increases_burst_dmg() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::RaidenShogun, CharId::Noelle, CharId::Fischl],
        vector![CharId::Ganyu, CharId::Xiangling, CharId::Xingqiu],
    )
    .with_enable_log(true)
    .with_ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::TranscendenceBalefulOmen),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Breastplate)),
    ]);
    // Not buffed
    assert_eq!(9, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SweepingTime),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_summon(SummonId::EyeOfStormyJudgment));
    // Buffed
    assert_eq!(4, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
}

#[test]
fn chakra_desiderata_buffs_burst() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Noelle, CharId::Fischl, CharId::RaidenShogun],
        vector![CharId::Ganyu, CharId::Xiangling, CharId::Xingqiu],
    )
    .with_enable_log(true)
    .with_ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(2, StatusId::ChakraDesiderata));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::SweepingTime)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::MidnightPhantasmagoria),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0].set_hp(10);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SecretArtMusouShinsetsu),
    )]);
    assert_eq!(5, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        0,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Character(2, StatusId::ChakraDesiderata))
            .unwrap()
            .get_counter()
    );
}

#[test]
fn chakra_desiderata_under_talent_card_buffs_burst_twice() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Noelle, CharId::Fischl, CharId::RaidenShogun],
        vector![CharId::Ganyu, CharId::Xiangling, CharId::Xingqiu],
    )
    .with_enable_log(true)
    .with_ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerFirst)
        .hand
        .push(CardId::WishesUnnumbered);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(2, StatusId::ChakraDesiderata));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::SweepingTime)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::MidnightPhantasmagoria),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0].set_hp(10);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::WishesUnnumbered, Some(CardSelection::OwnCharacter(2))),
    )]);
    assert_eq!(3, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
}

#[test]
fn chakra_desiderata_counter_not_increased() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::RaidenShogun], vector![CharId::Ganyu])
        .with_enable_log(true)
        .with_ignore_costs(true)
        .build();

    gs.advance_roll_phase_no_dice();
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_character_status(0, StatusId::ChakraDesiderata));
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SecretArtMusouShinsetsu),
    )]);
    assert_eq!(
        0,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Character(0, StatusId::ChakraDesiderata))
            .unwrap()
            .get_counter()
    );
}
