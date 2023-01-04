use super::*;

#[test]
fn test_cryo_infusion() {
    let mut gs = GameState::new(
        &vector![CharId::Yoimiya, CharId::KamisatoAyaka],
        &vector![CharId::Fischl, CharId::Ganyu],
        true,
    );
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
        ),
    ]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, fischl.get_hp());
    }
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
        ),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
    ]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(4, fischl.get_hp());
    }
    gs.advance_multiple(&vec![NO_ACTION]);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
        ),
    ]);
    {
        let kaeya = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![], kaeya.applied);
        assert_eq!(8, kaeya.get_hp());
    }
}

#[test]
fn test_cryo_infusion_at_duel_start() {
    let mut gs = GameState::new(&vector![CharId::KamisatoAyaka], &vector![CharId::Fischl], true);
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
    )]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(8, fischl.get_hp());
    }
}

#[test]
fn test_kamisato_art_soumetsu_summon() {
    let mut gs = GameState::new(&vector![CharId::KamisatoAyaka], &vector![CharId::Fischl], true);
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KamisatoArtSoumetsu),
    )]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(6, fischl.get_hp());
    }
    assert!(gs.players.0.has_summon(SummonId::FrostflakeSekiNoTo));

    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        NO_ACTION,
    ]);
    gs.advance_roll_phase_no_dice();
    assert_eq!(2, gs.round_number);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(4, fischl.get_hp());
    }
    assert!(gs.players.0.has_summon(SummonId::FrostflakeSekiNoTo));

    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        NO_ACTION,
    ]);
    gs.advance_roll_phase_no_dice();
    assert_eq!(3, gs.round_number);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(2, fischl.get_hp());
    }

    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        NO_ACTION,
    ]);
    gs.advance_roll_phase_no_dice();
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(2, fischl.get_hp());
    }
    assert_eq!(4, gs.round_number);
    assert!(!gs.players.0.has_summon(SummonId::FrostflakeSekiNoTo));
}

#[test]
fn test_cryo_infusion_under_talent_card() {
    let mut gs = GameState::new(
        &vector![CharId::Yoimiya, CharId::KamisatoAyaka],
        &vector![CharId::Fischl, CharId::Ganyu],
        true,
    );
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::KantenSenmyouBlessing);
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::KantenSenmyouBlessing, Some(CardSelection::OwnCharacter(1))),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
        ),
    ]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(7, fischl.get_hp());
    }
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
    )]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(5, fischl.get_hp());
    }
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        NO_ACTION,
    ]);
    gs.advance_roll_phase_no_dice();
    assert!(gs
        .players
        .0
        .status_collection
        .get(StatusKey::Character(1, StatusId::CryoElementalInfusion))
        .is_none());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
        ),
    ]);
    assert!(gs.players.0.char_states[1].has_talent_equipped());
    assert!(gs
        .players
        .0
        .status_collection
        .get(StatusKey::Character(1, StatusId::CryoElementalInfusion))
        .is_some());
    {
        let ganyu = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], ganyu.applied);
        assert_eq!(7, ganyu.get_hp());
    }
}
