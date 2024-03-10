use super::*;

#[test]
fn seed_of_skandha_receive_pd() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Nahida, CharId::Mona],
        vector![CharId::Ganyu, CharId::Fischl, CharId::Kaeya, CharId::Noelle],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::AllSchemesToKnowTathata),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    {
        for i in 0..=3 {
            assert!(gs.has_character_status(PlayerId::PlayerSecond, i, StatusId::SeedOfSkandha));
        }
    }
    // Artificially remove last status
    gs.status_collection_mut(PlayerId::PlayerSecond)
        .delete(StatusKey::Character(2, StatusId::SeedOfSkandha));
    {
        let p = gs.player(PlayerId::PlayerSecond);
        assert_eq!(7, p.char_states[0].hp());
    }
    gs.player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::RippleOfFate)),
    ]);
    {
        let p = gs.player(PlayerId::PlayerSecond);
        assert_eq!(
            vec![3, 9, 10, 9],
            p.char_states.iter_valid().map(CharState::hp).collect::<Vec<_>>()
        );
    }
}

#[test]
fn shrine_of_maya_increases_outgoing_reaction_dmg() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Nahida, CharId::Mona],
        vector![CharId::Ganyu, CharId::Fischl, CharId::Kaeya, CharId::Noelle],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::IllusoryHeart)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    gs.has_team_status(PlayerId::PlayerFirst, StatusId::ShrineOfMaya);
    assert_eq!(6, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    assert_eq!(
        elem_set![Element::Dendro],
        gs.player(PlayerId::PlayerSecond).char_states[0].applied
    );
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::RippleOfFate)),
    ]);
    assert_eq!(3, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    assert_eq!(elem_set![], gs.player(PlayerId::PlayerSecond).char_states[0].applied);
}
