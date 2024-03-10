use super::*;

#[test]
fn swirl_deals_swirl_dmg_and_applies_element() {
    for e in [Element::Pyro, Element::Hydro, Element::Electro, Element::Cryo] {
        let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
            vector![CharId::Sucrose],
            vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
        )
        .ignore_costs(true)
        .build();
        gs.player_mut(PlayerId::PlayerSecond).char_states[0].applied.insert(e);
        gs.advance_roll_phase_no_dice();
        gs.advance_multiple([Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
        )]);
        {
            let player = gs.player(PlayerId::PlayerSecond);
            assert_eq!(elem_set![], player.char_states[0].applied);
            assert_eq!(elem_set![e], player.char_states[1].applied);
            assert_eq!(elem_set![e], player.char_states[2].applied);
            assert_eq!(7, player.char_states[0].hp());
            assert_eq!(9, player.char_states[1].hp());
            assert_eq!(9, player.char_states[2].hp());
        }
    }
}

#[test]
fn swirl_triggers_secondary_reactions_melt_vaporize() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .ignore_costs(true)
    .build();
    {
        let player = gs.player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Pyro);
        player.char_states[1].applied.insert(Element::Cryo);
        player.char_states[2].applied.insert(Element::Hydro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    {
        let player = gs.player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![], player.char_states[2].applied);
        assert_eq!(7, player.char_states[0].hp());
        assert_eq!(7, player.char_states[1].hp());
        assert_eq!(7, player.char_states[2].hp());
    }
}

#[test]
fn swirl_triggers_secondary_reactions_electro_charged_superconduct() {
    for e in [Element::Hydro, Element::Cryo] {
        let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
            vector![CharId::Sucrose],
            vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
        )
        .ignore_costs(true)
        .build();
        {
            let player = gs.player_mut(PlayerId::PlayerSecond);
            player.char_states[0].applied.insert(Element::Electro);
            player.char_states[1].applied.insert(e);
        }
        gs.advance_roll_phase_no_dice();

        gs.advance_multiple([Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
        )]);
        {
            let player = gs.player(PlayerId::PlayerSecond);
            assert_eq!(elem_set![], player.char_states[0].applied);
            assert_eq!(elem_set![], player.char_states[1].applied);
            assert_eq!(elem_set![Element::Electro], player.char_states[2].applied);
            assert_eq!(6, player.char_states[0].hp());
            assert_eq!(8, player.char_states[1].hp());
            assert_eq!(8, player.char_states[2].hp());
        }
    }
}

#[test]
fn swirl_triggers_secondary_reactions_bloom_frozen() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .ignore_costs(true)
    .build();
    {
        let player = gs.player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Hydro);
        player.char_states[1].applied.insert(Element::Cryo);
        player.char_states[2].applied.insert(Element::Dendro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert!(gs.has_team_status(PlayerId::PlayerFirst, StatusId::DendroCore));
    assert!(gs
        .status_collection_mut(PlayerId::PlayerSecond)
        .get(StatusKey::Character(1, StatusId::Frozen))
        .is_some());
    {
        let player = gs.player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![], player.char_states[2].applied);
    }
}

#[test]
fn swirl_triggers_secondary_reactions_quicken() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .ignore_costs(true)
    .build();
    {
        let player = gs.player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Electro);
        player.char_states[1].applied.insert(Element::Dendro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert!(gs.has_team_status(PlayerId::PlayerFirst, StatusId::CatalyzingField));
    {
        let player = gs.player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![Element::Electro], player.char_states[2].applied);
    }
}

#[test]
fn swirl_triggers_secondary_reactions_burning() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .ignore_costs(true)
    .build();
    {
        let player = gs.player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Pyro);
        player.char_states[1].applied.insert(Element::Dendro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::BurningFlame));
    {
        let player = gs.player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![Element::Pyro], player.char_states[2].applied);
    }
}

#[test]
fn swirl_triggers_secondary_reactions_overloaded_no_forced_switch() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Yoimiya, CharId::Fischl, CharId::Ganyu],
    )
    .ignore_costs(true)
    .build();
    {
        let player = gs.player_mut(PlayerId::PlayerSecond);
        player.char_states[0].applied.insert(Element::Pyro);
        player.char_states[1].applied.insert(Element::Electro);
        player.char_states[2].applied.insert(Element::Electro);
    }
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple([
        // Different skill from above
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ForbiddenCreationIsomer75TypeII),
        ),
    ]);
    {
        let player = gs.player(PlayerId::PlayerSecond);
        assert_eq!(elem_set![], player.char_states[0].applied);
        assert_eq!(elem_set![], player.char_states[1].applied);
        assert_eq!(elem_set![], player.char_states[2].applied);
        assert_eq!(0, player.active_char_idx);
    }
}
