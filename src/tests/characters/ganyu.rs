use super::*;

#[test]
fn test_frostflake_arrow_piercing_dmg() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Ganyu],
        vector![CharId::Fischl, CharId::Yoimiya, CharId::Kaeya],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FrostflakeArrow),
    )]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, fischl.get_hp());
        assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
        assert_eq!(
            elem_set![],
            gs.get_player(PlayerId::PlayerSecond).char_states[1].applied
        );
        assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[2].get_hp());
        assert_eq!(
            elem_set![],
            gs.get_player(PlayerId::PlayerSecond).char_states[2].applied
        );
    }
}

#[test]
fn test_talent_card_does_not_increase_frostflake_arrow_dmg_first_cast() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Ganyu],
        vector![CharId::Fischl, CharId::Yoimiya, CharId::Kaeya],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    gs.players.0.hand.push(CardId::UndividedHeart);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::UndividedHeart, Some(CardSelection::OwnCharacter(0))),
    )]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, fischl.get_hp());
        assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
        assert_eq!(
            elem_set![],
            gs.get_player(PlayerId::PlayerSecond).char_states[1].applied
        );
        assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[2].get_hp());
        assert_eq!(
            elem_set![],
            gs.get_player(PlayerId::PlayerSecond).char_states[2].applied
        );
    }
}

#[test]
fn test_talent_card_increases_frostflake_arrow_dmg_subsequent_cast() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Ganyu],
        vector![CharId::Fischl, CharId::Yoimiya, CharId::Kaeya],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    gs.players.0.hand.push(CardId::UndividedHeart);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FrostflakeArrow)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
        assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
        assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[2].get_hp());
    }
    assert_eq!(2, gs.round_number);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::UndividedHeart, Some(CardSelection::OwnCharacter(0))),
        ),
    ]);
    {
        assert_eq!(5, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
        assert_eq!(5, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
        assert_eq!(5, gs.get_player(PlayerId::PlayerSecond).char_states[2].get_hp());
    }
}
