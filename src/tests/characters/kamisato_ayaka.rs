use super::*;

#[test]
fn cryo_infusion() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::KamisatoAyaka],
        vector![CharId::Fischl, CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
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
    gs.advance_multiple([
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
    gs.advance_multiple([NO_ACTION]);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
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
fn cryo_infusion_at_duel_start() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::KamisatoAyaka], vector![CharId::Fischl])
        .enable_log(true)
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
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
fn kamisato_art_soumetsu_summon() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::KamisatoAyaka], vector![CharId::Fischl])
        .enable_log(true)
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KamisatoArtSoumetsu),
    )]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(6, fischl.get_hp());
    }
    assert!(gs.players.0.has_summon(SummonId::FrostflakeSekiNoTo));

    gs.advance_multiple([
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

    gs.advance_multiple([
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

    gs.advance_multiple([
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
fn cryo_infusion_under_talent_card() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::KamisatoAyaka],
        vector![CharId::Fischl, CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.add_to_hand_ignore(CardId::KantenSenmyouBlessing);
    gs.advance_multiple([
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
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
    )]);
    {
        let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Cryo], fischl.applied);
        assert_eq!(5, fischl.get_hp());
    }
    gs.advance_multiple([
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
    gs.advance_multiple([
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
