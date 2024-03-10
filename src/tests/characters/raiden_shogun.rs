use super::*;

#[test]
fn secret_art_musou_shinsetsu_increases_energy() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::RaidenShogun, CharId::Noelle, CharId::Fischl],
        vector![CharId::Ganyu, CharId::Xiangling, CharId::Xingqiu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SecretArtMusouShinsetsu),
    )]);
    assert_eq!(
        vec![0, 2, 2],
        gs.player(PlayerId::PlayerFirst)
            .char_states
            .iter_valid()
            .map(CharState::energy)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec![0, 0, 0],
        gs.player(PlayerId::PlayerSecond)
            .char_states
            .iter_valid()
            .map(CharState::energy)
            .collect::<Vec<_>>()
    );
}

#[test]
fn eye_of_stormy_judgment_increases_burst_dmg() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::RaidenShogun, CharId::Noelle, CharId::Fischl],
        vector![CharId::Ganyu, CharId::Xiangling, CharId::Xingqiu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
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
    assert_eq!(9, gs.player(PlayerId::PlayerSecond).char_states[1].hp());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SweepingTime),
    )]);
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::EyeOfStormyJudgment));
    // Buffed
    assert_eq!(4, gs.player(PlayerId::PlayerSecond).char_states[1].hp());
}

#[test]
fn chakra_desiderata_buffs_burst() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Noelle, CharId::Fischl, CharId::RaidenShogun],
        vector![CharId::Ganyu, CharId::Xiangling, CharId::Xingqiu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    assert!(gs.has_character_status(PlayerId::PlayerFirst, 2, StatusId::ChakraDesiderata));
    gs.advance_multiple([
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
    gs.player_mut(PlayerId::PlayerSecond).char_states[0].set_hp(10);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SecretArtMusouShinsetsu),
    )]);
    assert_eq!(5, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    assert_eq!(
        0,
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Character(2, StatusId::ChakraDesiderata))
            .unwrap()
            .counter()
    );
}

#[test]
fn chakra_desiderata_under_talent_card_buffs_burst_twice() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Noelle, CharId::Fischl, CharId::RaidenShogun],
        vector![CharId::Ganyu, CharId::Xiangling, CharId::Xingqiu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.player_mut(PlayerId::PlayerFirst)
        .add_to_hand_ignore(CardId::WishesUnnumbered);
    assert!(gs.has_character_status(PlayerId::PlayerFirst, 2, StatusId::ChakraDesiderata));
    gs.advance_multiple([
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
    gs.player_mut(PlayerId::PlayerSecond).char_states[0].set_hp(10);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::WishesUnnumbered, Some(CardSelection::OwnCharacter(2))),
    )]);
    assert_eq!(3, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
}

#[test]
fn chakra_desiderata_counter_not_increased() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::RaidenShogun], vector![CharId::Ganyu])
            .enable_log(true)
            .ignore_costs(true)
            .build();

    gs.advance_roll_phase_no_dice();
    assert!(gs.has_character_status(PlayerId::PlayerFirst, 0, StatusId::ChakraDesiderata));
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::SecretArtMusouShinsetsu),
    )]);
    assert_eq!(
        0,
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Character(0, StatusId::ChakraDesiderata))
            .unwrap()
            .counter()
    );
}
