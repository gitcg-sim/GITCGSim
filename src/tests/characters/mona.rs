use super::*;

#[test]
fn passive_switch_character_away_is_fast_action_once_per_round() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Mona, CharId::Kaeya], vector![CharId::Fischl])
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
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
    gs.advance_multiple([
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
fn reflection_expires_without_usage_being_consumed() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Mona], vector![CharId::Fischl])
            .ignore_costs(true)
            .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::MirrorReflectionOfDoom),
    )]);
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::Reflection));
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    gs.advance_roll_phase_no_dice();
    assert!(!gs.has_summon(PlayerId::PlayerFirst, SummonId::Reflection));
    let fischl = gs.player(PlayerId::PlayerSecond).active_character();
    assert_eq!(elem_set![Element::Hydro], fischl.applied);
    assert_eq!(8, fischl.hp());
}

#[test]
fn reflection_reduces_dmg_and_remains_until_end_phase() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Mona], vector![CharId::Fischl])
            .ignore_costs(true)
            .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::MirrorReflectionOfDoom),
    )]);
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::Reflection));
    assert_eq!(
        1,
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::Reflection))
            .unwrap()
            .usages()
    );
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
    )]);
    assert_eq!(
        0,
        gs.status_collection_mut(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::Reflection))
            .unwrap()
            .usages()
    );
    assert_eq!(9, gs.player(PlayerId::PlayerFirst).active_character().hp());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    gs.player_mut(PlayerId::PlayerSecond).char_states[0].applied.clear();
    gs.advance_multiple([Input::NoAction]);
    gs.advance_roll_phase_no_dice();
    assert!(!gs.has_summon(PlayerId::PlayerFirst, SummonId::Reflection));
    let fischl = gs.player(PlayerId::PlayerSecond).active_character();
    assert_eq!(elem_set![Element::Hydro], fischl.applied);
    assert_eq!(8, fischl.hp());
}

// TODO do reaction DMG bonuses apply before or after doubling? (after currently)
#[test]
fn stellaris_phantasm_doubles_dmg() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Mona, CharId::Xingqiu],
        vector![CharId::Fischl, CharId::Barbara],
    )
    .build();
    gs.advance_roll_phase_no_dice();
    gs.ignore_costs = true;
    gs.player_mut(PlayerId::PlayerFirst)
        .add_to_hand_ignore(CardId::SacrificialSword);
    gs.advance_multiple([
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
    assert_eq!(4, gs.player(PlayerId::PlayerSecond).active_character().hp());

    assert!(!gs.has_team_status(PlayerId::PlayerFirst, StatusId::IllusoryBubble));
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuhuaStyle)),
    ]);

    assert_eq!(1, gs.player(PlayerId::PlayerSecond).active_character().hp());
}

#[test]
fn stellaris_phantasm_doubles_dmg_for_reaction_post_reaction_bonus() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Mona, CharId::Bennett],
        vector![CharId::Fischl, CharId::Barbara],
    )
    .build();
    gs.advance_roll_phase_no_dice();
    gs.ignore_costs = true;
    gs.player_mut(PlayerId::PlayerSecond).char_states[1]
        .applied
        .insert(Element::Hydro);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::StellarisPhantasm),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::PassionOverload)),
    ]);
    // Vaporize: 10 - 2*(3 + 2) = 0
    assert_eq!(0, gs.player(PlayerId::PlayerSecond).char_states[1].hp());
    assert!(!gs.has_team_status(PlayerId::PlayerFirst, StatusId::IllusoryBubble));
}

#[test]
fn stellaris_phantasm_does_not_double_summon_dmg() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Xiangling, CharId::Mona],
        vector![CharId::Fischl, CharId::Barbara, CharId::Kaeya],
    )
    .build();
    gs.advance_roll_phase_no_dice();
    gs.ignore_costs = true;
    gs.advance_multiple([
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
    assert!(gs.has_team_status(PlayerId::PlayerFirst, StatusId::IllusoryBubble));
    assert_eq!(6, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
    assert_eq!(10, gs.player(PlayerId::PlayerSecond).char_states[1].hp());
    assert_eq!(10, gs.player(PlayerId::PlayerSecond).char_states[2].hp());

    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);

    // 2 DMG taken
    assert_eq!(8, gs.player(PlayerId::PlayerSecond).char_states[2].hp());
    assert_eq!(
        elem_set![Element::Pyro],
        gs.player(PlayerId::PlayerSecond).char_states[2].applied
    );
}
