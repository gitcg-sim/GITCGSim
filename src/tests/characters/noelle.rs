use super::*;

#[test]
fn breastplate_shield_points() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Noelle], vector![CharId::Ganyu])
            .enable_log(true)
            .ignore_costs(true)
            .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
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
        let noelle = gs.player(PlayerId::PlayerFirst).active_character();
        assert_eq!(8, noelle.hp());
    }
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Breastplate),
    )]);
    assert!(gs.has_team_status(PlayerId::PlayerFirst, StatusId::FullPlate));

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::FrostflakeArrow),
    )]);
    {
        let noelle = gs.player(PlayerId::PlayerFirst).active_character();
        assert_eq!(8, noelle.hp());
    }
    assert!(!gs.has_team_status(PlayerId::PlayerFirst, StatusId::FullPlate));
}

#[test]
fn talent_card_heals_all() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Noelle, CharId::Yoimiya, CharId::Ganyu],
        vector![CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.players.0.add_to_hand_ignore(CardId::IGotYourBack);
    for c in gs.players.0.char_states.iter_all_mut() {
        c.set_hp(5)
    }

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
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

    assert!(gs.has_team_status(PlayerId::PlayerFirst, StatusId::FullPlate));

    for c in gs.players.0.char_states.iter_valid() {
        assert_eq!(6, c.hp())
    }

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
    )]);

    for c in gs.players.0.char_states.iter_valid() {
        assert_eq!(6, c.hp())
    }
}

#[test]
fn sweeping_time_reduces_cost_for_na() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Noelle], vector![CharId::Ganyu])
            .enable_log(true)
            .build();
    gs.advance_roll_phase_no_dice();
    {
        let p = gs.player_mut(PlayerId::PlayerFirst);
        p.dice.add_single(Dice::Omni, 7);
        p.dice.add_single(Dice::DENDRO, 1);
        p.char_states[0].set_energy(3);
    }

    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::SweepingTime)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(4, gs.player(PlayerId::PlayerFirst).dice.total());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
    )]);
    assert_eq!(2, gs.player(PlayerId::PlayerFirst).dice.total());
    assert!(gs
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ))
        .is_err());
}
