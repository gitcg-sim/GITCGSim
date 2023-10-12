use super::*;

#[test]
fn test_seed_of_skandha_receive_pd() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Nahida, CharId::Mona],
        vector![CharId::Ganyu, CharId::Fischl, CharId::Kaeya, CharId::Noelle],
    )
    .with_enable_log(true)
    .with_ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::AllSchemesToKnowTathata),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    {
        let p = gs.get_player(PlayerId::PlayerSecond);
        for i in 0..=3 {
            assert!(p.has_character_status(i, StatusId::SeedOfSkandha));
        }
    }
    // Artificially remove last status
    gs.get_player_mut(PlayerId::PlayerSecond)
        .status_collection
        .delete(StatusKey::Character(2, StatusId::SeedOfSkandha));
    {
        let p = gs.get_player(PlayerId::PlayerSecond);
        assert_eq!(7, p.char_states[0].get_hp());
    }
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::RippleOfFate)),
    ]);
    {
        let p = gs.get_player(PlayerId::PlayerSecond);
        assert_eq!(
            vec![3, 9, 10, 9],
            p.char_states.iter().map(|c| c.get_hp()).collect::<Vec<_>>()
        );
    }
}

// TODO test Klee self-DMG

#[test]
fn test_shrine_of_maya_increases_outgoing_reaction_dmg() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Nahida, CharId::Mona],
        vector![CharId::Ganyu, CharId::Fischl, CharId::Kaeya, CharId::Noelle],
    )
    .with_enable_log(true)
    .with_ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::IllusoryHeart)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    gs.get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::ShrineOfMaya);
    assert_eq!(6, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        elem_set![Element::Dendro],
        gs.get_player(PlayerId::PlayerSecond).char_states[0].applied
    );
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::RippleOfFate)),
    ]);
    assert_eq!(3, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        elem_set![],
        gs.get_player(PlayerId::PlayerSecond).char_states[0].applied
    );
}
