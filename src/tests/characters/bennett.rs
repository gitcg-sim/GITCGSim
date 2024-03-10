use super::*;

#[test]
fn inspiration_field_dmg_bonus_and_no_heal_for_character_above_7hp() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Bennett, CharId::Ganyu],
        vector![CharId::Fischl, CharId::Noelle],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.player_mut(PlayerId::PlayerFirst).char_states[1].set_hp(7);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FantasticVoyage)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrailOfTheQilin)),
    ]);
    assert_eq!(7, gs.player(PlayerId::PlayerFirst).char_states[1].hp());
    assert_eq!(7, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
}

#[test]
fn inspiration_field_no_dmg_bonus_and_heals_for_character_above_7hp() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Bennett, CharId::Ganyu],
        vector![CharId::Fischl, CharId::Noelle],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.player_mut(PlayerId::PlayerFirst).char_states[1].set_hp(6);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FantasticVoyage)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrailOfTheQilin)),
    ]);
    assert_eq!(8, gs.player(PlayerId::PlayerFirst).char_states[1].hp());
    assert_eq!(9, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
}

#[test]
fn talent_card_has_dmg_bonus_and_no_heal_for_character_above_7hp() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Bennett, CharId::Ganyu],
        vector![CharId::Fischl, CharId::Noelle],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    {
        let p = gs.player_mut(PlayerId::PlayerFirst);
        p.add_to_hand_ignore(CardId::GrandExpectation);
        p.char_states[1].set_hp(7);
    }
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::GrandExpectation, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrailOfTheQilin)),
    ]);
    assert_eq!(7, gs.player(PlayerId::PlayerFirst).char_states[1].hp());
    assert_eq!(7, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
}

#[test]
fn talent_card_has_dmg_bonus_and_heals_for_character_below_7hp() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Bennett, CharId::Ganyu],
        vector![CharId::Fischl, CharId::Noelle],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    {
        let p = gs.player_mut(PlayerId::PlayerFirst);
        p.add_to_hand_ignore(CardId::GrandExpectation);
        p.char_states[1].set_hp(6);
    }
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::GrandExpectation, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrailOfTheQilin)),
    ]);
    assert_eq!(8, gs.player(PlayerId::PlayerFirst).char_states[1].hp());
    assert_eq!(7, gs.player(PlayerId::PlayerSecond).char_states[0].hp());
}
