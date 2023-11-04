use super::*;

#[test]
fn fatal_rainscreen_applies_hydro_to_self_and_creates_rain_sword() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Xingqiu], vector![CharId::Fischl])
        .with_ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FatalRainscreen),
    )]);
    assert_eq!(
        elem_set![Element::Hydro],
        gs.get_player(PlayerId::PlayerFirst).get_active_character().applied
    );
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::RainSword));
    assert_eq!(
        elem_set![Element::Hydro],
        gs.get_player(PlayerId::PlayerSecond).get_active_character().applied
    );
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        2,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Team(StatusId::RainSword))
            .unwrap()
            .get_usages()
    );
}

#[test]
fn talent_card_increases_fatal_rainscreen_usages() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Xingqiu], vector![CharId::Fischl])
        .with_ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::TheScentRemained);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::TheScentRemained, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(
        elem_set![Element::Hydro],
        gs.get_player(PlayerId::PlayerFirst).get_active_character().applied
    );
    assert_eq!(
        3,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Team(StatusId::RainSword))
            .unwrap()
            .get_usages()
    );
}

#[test]
fn rain_sword_reduces_dmg_above_3_by_1() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Xingqiu, CharId::Yoimiya],
        vector![CharId::Fischl, CharId::Noelle],
    )
    .with_ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FatalRainscreen)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
    ]);
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        2,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Team(StatusId::RainSword))
            .unwrap()
            .get_usages()
    );
    assert_eq!(10, gs.get_player(PlayerId::PlayerFirst).char_states[1].get_hp());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::MidnightPhantasmagoria),
        ),
    ]);
    assert_eq!(6, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    assert_eq!(7, gs.get_player(PlayerId::PlayerFirst).char_states[1].get_hp());
    assert_eq!(
        1,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Team(StatusId::RainSword))
            .unwrap()
            .get_usages()
    );
}

#[test]
fn raincutter_applies_hydro_to_self_and_creates_rainbow_bladework() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Xingqiu], vector![CharId::Fischl])
        .with_ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Raincutter),
    )]);
    assert_eq!(
        elem_set![Element::Hydro],
        gs.get_player(PlayerId::PlayerFirst).get_active_character().applied
    );
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::RainbowBladework));
    assert_eq!(
        elem_set![Element::Hydro],
        gs.get_player(PlayerId::PlayerSecond).get_active_character().applied
    );
    assert_eq!(9, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        3,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Team(StatusId::RainbowBladework))
            .unwrap()
            .get_usages()
    );
}

#[test]
fn rainbow_bladework_procs_on_normal_attacks() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Xingqiu, CharId::Xiangling],
        vector![CharId::Fischl, CharId::Noelle],
    )
    .with_ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Raincutter)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuobaAttack)),
    ]);
    assert_eq!(
        3,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Team(StatusId::RainbowBladework))
            .unwrap()
            .get_usages()
    );
    assert_eq!(
        elem_set![],
        gs.get_player(PlayerId::PlayerSecond).get_active_character().applied
    );
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::DoughFu)),
    ]);
    assert_eq!(
        2,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Team(StatusId::RainbowBladework))
            .unwrap()
            .get_usages()
    );
    assert_eq!(
        elem_set![Element::Hydro],
        gs.get_player(PlayerId::PlayerSecond).get_active_character().applied
    );
}
