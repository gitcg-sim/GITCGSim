use super::*;

#[test]
fn golden_chalices_bounty_generates_bountiful_core_when_meeting_requirements() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Nilou, CharId::Nahida, CharId::Mona],
        vector![CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::DanceOfHaftkarsvar),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs.has_team_status(PlayerId::PlayerFirst, StatusId::GoldenChalicesBounty));
}

#[test]
fn when_team_has_non_dendro_or_hydro_chars_golden_chalices_bounty_does_not_generate_bountiful_core() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Nilou, CharId::Noelle, CharId::Mona],
        vector![CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::DanceOfHaftkarsvar),
    )]);
    assert!(!gs.has_team_status(PlayerId::PlayerFirst, StatusId::GoldenChalicesBounty));
}

fn gs_bountiful_core() -> GameState {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Nilou, CharId::Nahida, CharId::Mona],
        vector![CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::DanceOfHaftkarsvar),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Akara)),
    ]);
    gs
}

#[test]
fn golden_chalices_bounty_generates_bountiful_cores() {
    let gs = gs_bountiful_core();
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::BountifulCore));
}

#[test]
fn bountiful_cores_deals_dmg_end_phase() {
    let mut gs = gs_bountiful_core();
    assert_eq!(5, gs.player(PlayerId::PlayerSecond).active_character().hp());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert_eq!(3, gs.player(PlayerId::PlayerSecond).active_character().hp());
    assert!(!gs.has_summon(PlayerId::PlayerFirst, SummonId::BountifulCore));
}

#[test]
fn bountiful_cores_deals_dmg_end_of_round_given_usages() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Nilou, CharId::Nahida, CharId::Mona],
        vector![CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::DanceOfHaftkarsvar),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::LiutianArchery)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::LiutianArchery)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Akara)),
    ]);
    {
        let bountiful_core = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get_mut(StatusKey::Summon(SummonId::BountifulCore))
            .unwrap();
        bountiful_core.set_usages(3);
    }
    assert_eq!(5, gs.player(PlayerId::PlayerSecond).active_character().hp());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::LiutianArchery)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
    ]);
    assert_eq!(3, gs.player(PlayerId::PlayerSecond).active_character().hp());
    {
        let bountiful_core = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get_mut(StatusKey::Summon(SummonId::BountifulCore))
            .unwrap();
        assert_eq!(2, bountiful_core.usages());
    }
}

#[test]
fn talent_card_increases_bountiful_core_dmg() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Nilou, CharId::Nahida, CharId::Mona],
        vector![CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.player_mut(PlayerId::PlayerFirst)
        .add_to_hand_ignore(CardId::TheStarrySkiesTheirFlowersRain);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(
                CardId::TheStarrySkiesTheirFlowersRain,
                Some(CardSelection::OwnCharacter(0)),
            ),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::LiutianArchery)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::LiutianArchery)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Akara)),
    ]);
    {
        let bountiful_core = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get_mut(StatusKey::Summon(SummonId::BountifulCore))
            .unwrap();
        bountiful_core.set_usages(3);
    }
    assert_eq!(5, gs.player(PlayerId::PlayerSecond).active_character().hp());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::LiutianArchery)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
    ]);
    gs.player_mut(PlayerId::PlayerSecond).active_character_mut().set_hp(5);
    {
        let bountiful_core = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get_mut(StatusKey::Summon(SummonId::BountifulCore))
            .unwrap();
        assert_eq!(2, bountiful_core.usages());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert_eq!(2, gs.player(PlayerId::PlayerSecond).active_character().hp());
    {
        let bountiful_core = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get_mut(StatusKey::Summon(SummonId::BountifulCore))
            .unwrap();
        assert_eq!(1, bountiful_core.usages());
    }
}

#[test]
fn lingering_aeon_receives_dmg_end_phase() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Nilou, CharId::Nahida, CharId::Mona],
        vector![CharId::Nilou],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::DanceOfAbzendegiDistantDreamsListeningSpring),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
    ]);
    assert!(gs.has_character_status(PlayerId::PlayerSecond, 0, StatusId::LingeringAeon));
    {
        let ganyu = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(8, ganyu.hp());
        assert_eq!(elem_set![Element::Hydro], ganyu.applied);
    }

    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert!(!gs.has_character_status(PlayerId::PlayerSecond, 0, StatusId::LingeringAeon));
    {
        let ganyu = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(5, ganyu.hp());
        assert_eq!(elem_set![Element::Hydro], ganyu.applied);
    }
}
