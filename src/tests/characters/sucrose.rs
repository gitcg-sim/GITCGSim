use super::*;

#[test]
fn astable_anemohypostasis_creation_6308_forces_switch_1_character() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Sucrose], vector![CharId::Ganyu])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert_eq!(0, gs.player(PlayerId::PlayerSecond).active_char_idx);
    assert!(gs.player(PlayerId::PlayerSecond).active_character().applied.is_empty());
}

#[test]
fn astable_anemohypostasis_creation_6308_forces_switch_to_prev() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Sucrose],
        vector![CharId::Ganyu, CharId::Yoimiya, CharId::Fischl],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert_eq!(2, gs.player(PlayerId::PlayerSecond).active_char_idx);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
        ),
    ]);
    assert_eq!(1, gs.player(PlayerId::PlayerSecond).active_char_idx);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert_eq!(0, gs.player(PlayerId::PlayerSecond).active_char_idx);
    gs.player_mut(PlayerId::PlayerSecond).char_states[2].set_hp(0);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::AstableAnemohypostasisCreation6308),
    )]);
    assert_eq!(1, gs.player(PlayerId::PlayerSecond).active_char_idx);
}

#[test]
fn large_wind_spirit_deals_anemo_dmg_without_infusion() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Sucrose], vector![CharId::Ganyu, CharId::Yoimiya])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::ForbiddenCreationIsomer75TypeII),
    )]);
    assert_eq!(9, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::LargeWindSpirit));
    {
        let summon_state = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::LargeWindSpirit))
            .unwrap();
        assert_eq!(3, summon_state.usages());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert_eq!(7, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    {
        let summon_state = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::LargeWindSpirit))
            .unwrap();
        assert_eq!(2, summon_state.usages());
    }
}

#[test]
fn large_wind_spirit_deals_infuses_after_swirling() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Sucrose], vector![CharId::Ganyu, CharId::Yoimiya])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::ForbiddenCreationIsomer75TypeII),
    )]);
    assert_eq!(9, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::LargeWindSpirit));
    gs.player_mut(PlayerId::PlayerSecond)
        .try_get_character_mut(0)
        .unwrap()
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert_eq!(7, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    assert_eq!(elem_set![], gs.player(PlayerId::PlayerSecond).char_states[0].applied);
    assert_eq!(
        elem_set![Element::Pyro],
        gs.player(PlayerId::PlayerSecond).char_states[1].applied
    );
    {
        let summon_state = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::LargeWindSpirit))
            .unwrap();
        assert_eq!(2, summon_state.usages());
        assert_eq!(Element::Pyro, Element::VALUES[summon_state.counter() as usize]);
    }
    gs.advance_roll_phase_no_dice();
    assert_eq!(2, gs.round_number);
    gs.player_mut(PlayerId::PlayerSecond)
        .try_get_character_mut(0)
        .unwrap()
        .applied
        .insert(Element::Cryo);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    {
        let summon_state = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::LargeWindSpirit))
            .unwrap();
        assert_eq!(1, summon_state.usages());
        // Did not re-infuse
        assert_eq!(Element::Pyro, Element::VALUES[summon_state.counter() as usize]);
    }
    assert_eq!(3, gs.round_number);
}

// TODO test different own summon swirling

#[test]
fn large_wind_spirit_infused_dmg_after_own_character_swirling() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Sucrose, CharId::Jean],
        vector![CharId::Jean, CharId::Yoimiya],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::ForbiddenCreationIsomer75TypeII),
    )]);
    assert_eq!(9, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::LargeWindSpirit));
    gs.player_mut(PlayerId::PlayerSecond)
        .try_get_character_mut(0)
        .unwrap()
        .applied
        .insert(Element::Pyro);
    {
        let mut gs = gs.clone();
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
            Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::CastSkill(SkillId::WindSpiritCreation),
            ),
        ]);
        {
            let summon_state = gs
                .status_collection_mut(PlayerId::PlayerFirst)
                .get(StatusKey::Summon(SummonId::LargeWindSpirit))
                .unwrap();
            assert_eq!(Element::Pyro, Element::VALUES[summon_state.counter() as usize]);
        }
    }

    {
        let mut gs = gs.clone();
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GaleBlade)),
        ]);
        {
            let summon_state = gs
                .status_collection_mut(PlayerId::PlayerFirst)
                .get(StatusKey::Summon(SummonId::LargeWindSpirit))
                .unwrap();
            assert_eq!(Element::Pyro, Element::VALUES[summon_state.counter() as usize]);
        }
    }
}

#[test]
fn large_wind_spirit_does_not_infuse_after_opponent_summon_swirling() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Sucrose], vector![CharId::Sucrose])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ForbiddenCreationIsomer75TypeII),
        ),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::ForbiddenCreationIsomer75TypeII),
        ),
    ]);
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::LargeWindSpirit));
    assert!(gs.has_summon(PlayerId::PlayerSecond, SummonId::LargeWindSpirit));
    gs.player_mut(PlayerId::PlayerFirst)
        .try_get_character_mut(0)
        .unwrap()
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert_eq!(elem_set![], gs.player(PlayerId::PlayerFirst).char_states[0].applied);
    assert_eq!(elem_set![], gs.player(PlayerId::PlayerSecond).char_states[0].applied);
    {
        let summon_state = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::LargeWindSpirit))
            .unwrap();
        // Opponent Swirled Pyro
        assert_eq!(Element::Anemo, Element::VALUES[summon_state.counter() as usize]);
    }
    {
        let summon_state = gs
            .status_collection_mut(PlayerId::PlayerSecond)
            .get(StatusKey::Summon(SummonId::LargeWindSpirit))
            .unwrap();
        assert_eq!(Element::Pyro, Element::VALUES[summon_state.counter() as usize]);
    }
}

#[test]
fn large_wind_spirit_does_not_infuse_after_opponent_skill_swirling() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Sucrose], vector![CharId::Sucrose])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::ForbiddenCreationIsomer75TypeII),
    )]);
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::LargeWindSpirit));
    gs.player_mut(PlayerId::PlayerFirst)
        .try_get_character_mut(0)
        .unwrap()
        .applied
        .insert(Element::Pyro);
    // Opponent Swirled Pyro
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::WindSpiritCreation),
    )]);
    {
        let summon_state = gs
            .status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::LargeWindSpirit))
            .unwrap();
        assert_eq!(Element::Anemo, Element::VALUES[summon_state.counter() as usize]);
    }
}
