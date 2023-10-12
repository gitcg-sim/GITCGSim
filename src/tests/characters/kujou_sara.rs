use super::*;

#[test]
fn test_tengu_juurai_ambush_applies_crowfeather_cover_end_phase() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(
            vector![CharId::KujouSara, CharId::Kaeya],
            vector![CharId::Ganyu, CharId::Xiangling],
        )
        .with_enable_log(true)
        .build()
    };
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::TenguStormcall),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Summon(SummonId::TenguJuuraiAmbush))
        .is_some());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    gs.advance_roll_phase_no_dice();
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Character(1, StatusId::CrowfeatherCover))
        .is_some());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::CeremonialBladework),
        ),
    ]);
    // Normal Attack: Usage not consumed
    assert_eq!(
        1,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Character(1, StatusId::CrowfeatherCover))
            .unwrap()
            .get_usages()
    );
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp(),);
    let gs0 = gs.clone();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Frostgnaw)),
    ]);

    // Elemental Skill: Usage consumed
    assert_eq!(4, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp(),);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Character(1, StatusId::CrowfeatherCover))
        .is_none());

    let mut gs = gs0;
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GlacialWaltz)),
    ]);

    // Elemental Burst: Usage consumed
    assert_eq!(6, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp(),);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Character(1, StatusId::CrowfeatherCover))
        .is_none());
}
