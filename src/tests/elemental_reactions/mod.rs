use enumset::enum_set;

use super::*;

#[test]
fn quicken() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Collei, CharId::Fischl],
        vector![CharId::Yoimiya, CharId::Fischl],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    {
        let yoimiya = &mut gs.players.1.char_states[0];
        yoimiya.applied |= Element::Electro;
    }
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FloralBrush),
    )]);
    assert!(gs.has_team_status(PlayerId::PlayerFirst, StatusId::CatalyzingField));
    assert_eq!(
        2,
        gs.get_status_collection_mut(PlayerId::PlayerFirst).team_statuses_vec()[0]
            .state
            .get_usages()
    );
    assert_eq!(6, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FloralBrush)),
    ]);
    assert_eq!(
        1,
        gs.get_status_collection_mut(PlayerId::PlayerFirst).team_statuses_vec()[0]
            .state
            .get_usages()
    );
    assert_eq!(6, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
    ]);
    assert_eq!(
        1,
        gs.get_status_collection_mut(PlayerId::PlayerFirst).team_statuses_vec()[0]
            .state
            .get_usages()
    );
    assert_eq!(4, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
}

#[test]
fn burning_max_2_stacks() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Collei], vector![CharId::Fischl])
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FloralBrush)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(
        1,
        gs.get_status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::BurningFlame))
            .unwrap()
            .get_usages()
    );
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FloralBrush),
    )]);
    assert_eq!(
        2,
        gs.get_status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::BurningFlame))
            .unwrap()
            .get_usages()
    );
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FloralBrush),
    )]);
    assert_eq!(
        2,
        gs.get_status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::BurningFlame))
            .unwrap()
            .get_usages()
    );
}

#[test]
fn bloom_dendro_core_increases_summon_dmg() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Fischl, CharId::Collei],
        vector![CharId::Yoimiya, CharId::Xingqiu],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FatalRainscreen),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FloralBrush)),
    ]);

    assert_eq!(
        1,
        gs.get_status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Team(StatusId::DendroCore))
            .unwrap()
            .get_usages()
    );
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert!(gs
        .get_status_collection_mut(PlayerId::PlayerFirst)
        .get(StatusKey::Team(StatusId::DendroCore))
        .is_none());
}

mod frozen;

mod overloaded;

mod swirl;
