use enumset::enum_set;

use super::*;

#[test]
fn test_quicken() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Collei, CharId::Fischl],
        vector![CharId::Yoimiya, CharId::Fischl],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    {
        let yoimiya = &mut gs.players.1.char_states[0];
        yoimiya.applied |= Element::Electro;
    }
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FloralBrush),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::CatalyzingField));
    assert_eq!(
        2,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .team_statuses_vec()[0]
            .state
            .get_usages()
    );
    assert_eq!(6, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FloralBrush)),
    ]);
    assert_eq!(
        1,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .team_statuses_vec()[0]
            .state
            .get_usages()
    );
    assert_eq!(6, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
    ]);
    assert_eq!(
        1,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .team_statuses_vec()[0]
            .state
            .get_usages()
    );
    assert_eq!(4, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
}

#[test]
fn test_burning_max_2_stacks() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Collei], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FloralBrush)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(
        1,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Summon(SummonId::BurningFlame))
            .unwrap()
            .get_usages()
    );
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FloralBrush),
    )]);
    assert_eq!(
        2,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Summon(SummonId::BurningFlame))
            .unwrap()
            .get_usages()
    );
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0]
        .applied
        .insert(Element::Pyro);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FloralBrush),
    )]);
    assert_eq!(
        2,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Summon(SummonId::BurningFlame))
            .unwrap()
            .get_usages()
    );
}

#[test]
fn test_bloom_dendro_core_increases_summon_dmg() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Fischl, CharId::Collei],
        vector![CharId::Yoimiya, CharId::Xingqiu],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple(&vec![
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
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Team(StatusId::DendroCore))
            .unwrap()
            .get_usages()
    );
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Team(StatusId::DendroCore))
        .is_none());
}

mod frozen;

mod overloaded;

mod swirl;
