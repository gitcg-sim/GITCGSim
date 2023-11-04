use super::*;

#[test]
fn breastplate_shield_points() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Noelle], vector![CharId::Ganyu])
        .with_enable_log(true)
        .with_ignore_costs(true)
        .build();

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
fn talent_card_heals_all() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Noelle, CharId::Yoimiya, CharId::Ganyu],
        vector![CharId::Ganyu],
    )
    .with_enable_log(true)
    .with_ignore_costs(true)
    .build();

    gs.players.0.hand.push(CardId::IGotYourBack);
    for c in gs.players.0.char_states.iter_all_mut() {
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

    for c in gs.players.0.char_states.iter_valid() {
        assert_eq!(6, c.get_hp())
    }

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
    )]);

    for c in gs.players.0.char_states.iter_valid() {
        assert_eq!(6, c.get_hp())
    }
}

#[test]
fn sweeping_time_reduces_cost_for_na() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Noelle], vector![CharId::Ganyu])
        .with_enable_log(true)
        .build();
    gs.advance_roll_phase_no_dice();
    {
        let p = gs.get_player_mut(PlayerId::PlayerFirst);
        p.dice.add_single(Dice::Omni, 7);
        p.dice.add_single(Dice::DENDRO, 1);
        p.char_states[0].set_energy(3);
    }

    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::SweepingTime)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(4, gs.get_player(PlayerId::PlayerFirst).dice.total());
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
    )]);
    assert_eq!(2, gs.get_player(PlayerId::PlayerFirst).dice.total());
    assert!(gs
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ))
        .is_err());
}
