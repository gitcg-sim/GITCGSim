use super::*;

#[test]
fn test_passive_switch_character_away_is_fast_action_once_per_round() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Mona, CharId::Kaeya], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        // consumed
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    gs.advance_roll_phase_no_dice();
    assert_eq!(2, gs.round_number);
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
        // consumed
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
    ]);
}

#[test]
fn test_reflection_expires_without_usage_being_consumed() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Mona], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::MirrorReflectionOfDoom),
    )]);
    assert!(gs.get_player(PlayerId::PlayerFirst).has_summon(SummonId::Reflection));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    gs.advance_roll_phase_no_dice();
    assert!(!gs.get_player(PlayerId::PlayerFirst).has_summon(SummonId::Reflection));
    let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
    assert_eq!(elem_set![Element::Hydro], fischl.applied);
    assert_eq!(8, fischl.get_hp());
}

#[test]
fn test_reflection_reduces_dmg_and_remains_until_end_phase() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Mona], vector![CharId::Fischl])
        .with_enable_log(true)
        .build();
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::MirrorReflectionOfDoom),
    )]);
    assert!(gs.get_player(PlayerId::PlayerFirst).has_summon(SummonId::Reflection));
    assert_eq!(
        1,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Summon(SummonId::Reflection))
            .unwrap()
            .get_usages()
    );
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
    )]);
    assert_eq!(
        0,
        gs.get_player(PlayerId::PlayerFirst)
            .status_collection
            .get(StatusKey::Summon(SummonId::Reflection))
            .unwrap()
            .get_usages()
    );
    assert_eq!(9, gs.get_player(PlayerId::PlayerFirst).get_active_character().get_hp());
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[0].applied.clear();
    gs.advance_multiple(&vec![Input::NoAction]);
    gs.advance_roll_phase_no_dice();
    assert!(!gs.get_player(PlayerId::PlayerFirst).has_summon(SummonId::Reflection));
    let fischl = gs.get_player(PlayerId::PlayerSecond).get_active_character();
    assert_eq!(elem_set![Element::Hydro], fischl.applied);
    assert_eq!(8, fischl.get_hp());
}

// TODO do reaction DMG bonuses apply before or after doubling? (after currently)
#[test]
fn test_stellaris_phantasm_doubles_dmg() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Mona, CharId::Xingqiu],
        vector![CharId::Fischl, CharId::Barbara],
    )
    .with_enable_log(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.ignore_costs = true;
    gs.get_player_mut(PlayerId::PlayerFirst)
        .hand
        .push(CardId::SacrificialSword);
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::SacrificialSword, Some(CardSelection::OwnCharacter(1))),
        ),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::StellarisPhantasm),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuhuaStyle)),
    ]);
    // 10 - 2*(2 + 1) = 4
    assert_eq!(4, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());

    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::IllusoryBubble));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuhuaStyle)),
    ]);

    assert_eq!(1, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
}

#[test]
fn test_stellaris_phantasm_doubles_dmg_for_reaction_post_reaction_bonus() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Mona, CharId::Bennett],
        vector![CharId::Fischl, CharId::Barbara],
    )
    .with_enable_log(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.ignore_costs = true;
    gs.get_player_mut(PlayerId::PlayerSecond).char_states[1]
        .applied
        .insert(Element::Hydro);
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::StellarisPhantasm),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::PassionOverload)),
    ]);
    // Vaporize: 10 - 2*(3 + 2) = 0
    assert_eq!(0, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::IllusoryBubble));
}

#[test]
fn test_stellaris_phantasm_does_not_double_summon_dmg() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Xiangling, CharId::Mona],
        vector![CharId::Fischl, CharId::Barbara, CharId::Kaeya],
    )
    .with_enable_log(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.ignore_costs = true;
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuobaAttack)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::StellarisPhantasm),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
    ]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_team_status(StatusId::IllusoryBubble));
    assert_eq!(6, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(10, gs.get_player(PlayerId::PlayerSecond).char_states[1].get_hp());
    assert_eq!(10, gs.get_player(PlayerId::PlayerSecond).char_states[2].get_hp());

    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);

    // 2 DMG taken
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[2].get_hp());
    assert_eq!(
        elem_set![Element::Pyro],
        gs.get_player(PlayerId::PlayerSecond).char_states[2].applied
    );
}
