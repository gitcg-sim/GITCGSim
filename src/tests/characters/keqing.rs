use super::*;

fn game_state_after_stellar_restoration() -> GameState {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Keqing, CharId::Ganyu],
        vector![CharId::Fischl, CharId::Yoimiya],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::StellarRestoration),
    )]);
    gs
}

#[test]
fn stellar_restoration_creates_lightning_stiletto() {
    let gs = game_state_after_stellar_restoration();
    {
        let fischl = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(7, fischl.hp());
        assert_eq!(elem_set![Element::Electro], fischl.applied);
    }
    assert_eq!(1, gs.player(PlayerId::PlayerFirst).hand.len());
    assert!(gs
        .player(PlayerId::PlayerFirst)
        .hand
        .contains(&CardId::LightningStiletto));
}

#[test]
fn lightning_stiletto_switches_to_keqing_and_casts_skill() {
    let mut gs = game_state_after_stellar_restoration();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::LightningStiletto, None),
        ),
    ]);
    {
        let yoimiya = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(7, yoimiya.hp());
        assert_eq!(elem_set![Element::Electro], yoimiya.applied);
    }
    assert!(!gs
        .player(PlayerId::PlayerFirst)
        .hand
        .contains(&CardId::LightningStiletto));
    assert_eq!(
        CharId::Keqing,
        gs.player(PlayerId::PlayerFirst).active_character().char_id
    );
}

#[test]
fn lightning_stiletto_casts_skill_when_keqing_is_active() {
    let mut gs = game_state_after_stellar_restoration();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::LightningStiletto, None),
        ),
    ]);
    {
        let yoimiya = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(7, yoimiya.hp());
        assert_eq!(elem_set![Element::Electro], yoimiya.applied);
    }
    assert!(!gs
        .player(PlayerId::PlayerFirst)
        .hand
        .contains(&CardId::LightningStiletto));
    assert_eq!(
        CharId::Keqing,
        gs.player(PlayerId::PlayerFirst).active_character().char_id
    );
}

#[test]
fn lightning_stiletto_cannot_be_played_with_dead_keqing() {
    let mut gs = game_state_after_stellar_restoration();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    gs.player_mut(PlayerId::PlayerFirst).char_states[0].reduce_hp(10);
    assert!(gs
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::LightningStiletto, None)
        ))
        .is_err());
}

#[test]
fn stellar_restoration_grants_electro_infusion_by_consuming_lightning_stiletto_on_hand() {
    let mut gs = game_state_after_stellar_restoration();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::StellarRestoration),
        ),
    ]);
    assert!(!gs
        .player(PlayerId::PlayerFirst)
        .hand
        .contains(&CardId::LightningStiletto));
    let electro_infusion = gs
        .status_collection_mut(PlayerId::PlayerFirst)
        .get(StatusKey::Character(0, StatusId::ElectroInfusion))
        .unwrap();

    assert_eq!(2, electro_infusion.duration());
}

#[test]
fn talent_card_increases_electro_infusion_duration() {
    let mut gs = game_state_after_stellar_restoration();
    gs.players.0.add_to_hand_ignore(CardId::ThunderingPenance);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::ThunderingPenance, Some(CardSelection::OwnCharacter(0))),
        ),
    ]);
    assert!(!gs
        .player(PlayerId::PlayerFirst)
        .hand
        .contains(&CardId::LightningStiletto));
    let electro_infusion = gs
        .status_collection_mut(PlayerId::PlayerFirst)
        .get(StatusKey::Character(0, StatusId::ElectroInfusion))
        .unwrap();

    assert_eq!(3, electro_infusion.duration());
}
