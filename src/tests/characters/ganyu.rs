use super::*;

#[test]
fn frostflake_arrow_piercing_dmg() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Ganyu],
        vector![CharId::Fischl, CharId::Yoimiya, CharId::Kaeya],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FrostflakeArrow),
    )]);
    {
        let fischl = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, fischl.hp());
        assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[1].hp());
        assert_eq!(elem_set![], gs.player(PlayerId::PlayerSecond).char_states[1].applied);
        assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[2].hp());
        assert_eq!(elem_set![], gs.player(PlayerId::PlayerSecond).char_states[2].applied);
    }
}

#[test]
fn talent_card_does_not_increase_frostflake_arrow_dmg_first_cast() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Ganyu],
        vector![CharId::Fischl, CharId::Yoimiya, CharId::Kaeya],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.players.0.add_to_hand_ignore(CardId::UndividedHeart);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::UndividedHeart, Some(CardSelection::OwnCharacter(0))),
    )]);
    {
        let fischl = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, fischl.hp());
        assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[1].hp());
        assert_eq!(elem_set![], gs.player(PlayerId::PlayerSecond).char_states[1].applied);
        assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[2].hp());
        assert_eq!(elem_set![], gs.player(PlayerId::PlayerSecond).char_states[2].applied);
    }
}

#[test]
fn talent_card_increases_frostflake_arrow_dmg_subsequent_cast() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Ganyu],
        vector![CharId::Fischl, CharId::Yoimiya, CharId::Kaeya],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.players.0.add_to_hand_ignore(CardId::UndividedHeart);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FrostflakeArrow)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    {
        let fischl = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
        assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[1].hp());
        assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[2].hp());
    }
    assert_eq!(2, gs.round_number);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::UndividedHeart, Some(CardSelection::OwnCharacter(0))),
        ),
    ]);
    {
        assert_eq!(5, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
        assert_eq!(5, gs.player(PlayerId::PlayerSecond).char_states[1].hp());
        assert_eq!(5, gs.player(PlayerId::PlayerSecond).char_states[2].hp());
    }
}
