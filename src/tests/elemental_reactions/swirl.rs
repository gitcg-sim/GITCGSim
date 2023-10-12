use super::*;

#[test]
fn test_swirl_deals_swirl_dmg_and_applies_element() {
    for e in [Element::Pyro, Element::Hydro, Element::Electro, Element::Cryo] {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Sucrose],
            vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
        )
        .with_ignore_costs(true)
        .build();
        gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
            .applied
            .insert(e);
        gs.advance_roll_phase_no_dice();
        gs.advance_multiple(&vec![Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
        )]);
        {
            let player = gs.get_player(PlayerId::PlayerSecond);
            assert_eq!(elem_set![], player.char_states[0].applied);
            assert_eq!(elem_set![e], player.char_states[1].applied);
            assert_eq!(elem_set![e], player.char_states[2].applied);
            assert_eq!(7, player.char_states[0].get_hp());
            assert_eq!(9, player.char_states[1].get_hp());
            assert_eq!(9, player.char_states[2].get_hp());
        }
    }
}

#[test]
fn test_swirl_triggers_secondary_reactions_melt_vaporize() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .with_ignore_costs(true)
    .build();
    {
        let player = gs.get_player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Pyro);
        player.char_states[1].applied.insert(Element::Cryo);
        player.char_states[2].applied.insert(Element::Hydro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    {
        let player = gs.get_player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![], player.char_states[2].applied);
        assert_eq!(7, player.char_states[0].get_hp());
        assert_eq!(7, player.char_states[1].get_hp());
        assert_eq!(7, player.char_states[2].get_hp());
    }
}

#[test]
fn test_swirl_triggers_secondary_reactions_electro_charged_superconduct() {
    for e in [Element::Hydro, Element::Cryo] {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Sucrose],
            vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
        )
        .with_ignore_costs(true)
        .build();
        {
            let player = gs.get_player_mut(PlayerId::PlayerSecond);
            player.char_states[0].applied.insert(Element::Electro);
            player.char_states[1].applied.insert(e);
        }
        gs.advance_roll_phase_no_dice();

        gs.advance_multiple(&vec![Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
        )]);
        {
            let player = gs.get_player(PlayerId::PlayerSecond);
            assert_eq!(elem_set![], player.char_states[0].applied);
            assert_eq!(elem_set![], player.char_states[1].applied);
            assert_eq!(elem_set![Element::Electro], player.char_states[2].applied);
            assert_eq!(6, player.char_states[0].get_hp());
            assert_eq!(8, player.char_states[1].get_hp());
            assert_eq!(8, player.char_states[2].get_hp());
        }
    }
}

#[test]
fn test_swirl_triggers_secondary_reactions_bloom_frozen() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .with_ignore_costs(true)
    .build();
    {
        let player = gs.get_player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Hydro);
        player.char_states[1].applied.insert(Element::Cryo);
        player.char_states[2].applied.insert(Element::Dendro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::DendroCore));
    assert!(gs
        .get_player(PlayerId::PlayerSecond)
        .status_collection
        .get(StatusKey::Character(1, StatusId::Frozen))
        .is_some());
    {
        let player = gs.get_player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![], player.char_states[2].applied);
    }
}

#[test]
fn test_swirl_triggers_secondary_reactions_quicken() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .with_ignore_costs(true)
    .build();
    {
        let player = gs.get_player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Electro);
        player.char_states[1].applied.insert(Element::Dendro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::CatalyzingField));
    {
        let player = gs.get_player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![Element::Electro], player.char_states[2].applied);
    }
}

#[test]
fn test_swirl_triggers_secondary_reactions_burning() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .with_ignore_costs(true)
    .build();
    {
        let player = gs.get_player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Pyro);
        player.char_states[1].applied.insert(Element::Dendro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert!(gs.get_player(PlayerId::PlayerFirst).has_summon(SummonId::BurningFlame));
    {
        let player = gs.get_player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![Element::Pyro], player.char_states[2].applied);
    }
}

#[test]
fn test_swirl_triggers_secondary_reactions_overloaded_no_forced_switch() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .with_ignore_costs(true)
    .build();
    {
        let player = gs.get_player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Pyro);
        player.char_states[1].applied.insert(Element::Electro);
        player.char_states[2].applied.insert(Element::Electro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple(&vec![
        // Different skill from above
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ForbiddenCreationIsomer75TypeII),
        ),
    ]);
    {
        let player = gs.get_player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![], player.char_states[2].applied);
        assert_eq!(0, player.active_char_index);
    }
}
