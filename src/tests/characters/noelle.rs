use super::*;

#[test]
fn test_breastplate_shield_points() {
    let mut gs = GameState::new(&vector![CharId::Noelle], &vector![CharId::Ganyu], true);
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FrostflakeArrow),
        ),
    ]);
    {
        let noelle = gs.get_player(PlayerId::PlayerFirst).get_active_character();
        assert_eq!(8, noelle.get_hp());
    }
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Breastplate),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::FullPlate));

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::FrostflakeArrow),
    )]);
    {
        let noelle = gs.get_player(PlayerId::PlayerFirst).get_active_character();
        assert_eq!(8, noelle.get_hp());
    }
    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::FullPlate));
}

#[test]
fn test_talent_card_heals_all() {
    let mut gs = GameState::new(
        &vector![CharId::Noelle, CharId::Yoimiya, CharId::Ganyu],
        &vector![CharId::Ganyu],
        true,
    );
    gs.ignore_costs = true;

    gs.players.0.hand.push(CardId::IGotYourBack);
    for c in gs.players.0.char_states.iter_mut() {
        c.set_hp(5)
    }

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::IGotYourBack, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ),
    ]);

    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::FullPlate));

    for c in gs.players.0.char_states.iter() {
        assert_eq!(6, c.get_hp())
    }

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
    )]);

    for c in gs.players.0.char_states.iter() {
        assert_eq!(6, c.get_hp())
    }
}
