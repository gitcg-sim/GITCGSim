use super::*;

#[test]
fn test_inspiration_field_dmg_bonus_and_no_heal_for_character_above_7hp() {
    let mut gs = GameState::new(
        &vector![CharId::Bennett, CharId::Ganyu],
        &vector![CharId::Fischl, CharId::Noelle],
        true,
    );
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerFirst).char_states[1].set_hp(7);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FantasticVoyage)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrailOfTheQilin)),
    ]);
    assert_eq!(7, gs.get_player(PlayerId::PlayerFirst).char_states[1].get_hp());
    assert_eq!(7, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
}

#[test]
fn test_inspiration_field_no_dmg_bonus_and_heals_for_character_above_7hp() {
    let mut gs = GameState::new(
        &vector![CharId::Bennett, CharId::Ganyu],
        &vector![CharId::Fischl, CharId::Noelle],
        true,
    );
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerFirst).char_states[1].set_hp(6);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FantasticVoyage)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrailOfTheQilin)),
    ]);
    assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[1].get_hp());
    assert_eq!(9, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
}

#[test]
fn test_talent_card_has_dmg_bonus_and_no_heal_for_character_above_7hp() {
    let mut gs = GameState::new(
        &vector![CharId::Bennett, CharId::Ganyu],
        &vector![CharId::Fischl, CharId::Noelle],
        true,
    );
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    {
        let p = gs.get_player_mut(PlayerId::PlayerFirst);
        p.hand.push(CardId::GrandExpectation);
        p.char_states[1].set_hp(7);
    }
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::GrandExpectation, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrailOfTheQilin)),
    ]);
    assert_eq!(7, gs.get_player(PlayerId::PlayerFirst).char_states[1].get_hp());
    assert_eq!(7, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
}

#[test]
fn test_talent_card_has_dmg_bonus_and_heals_for_character_below_7hp() {
    let mut gs = GameState::new(
        &vector![CharId::Bennett, CharId::Ganyu],
        &vector![CharId::Fischl, CharId::Noelle],
        true,
    );
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    {
        let p = gs.get_player_mut(PlayerId::PlayerFirst);
        p.hand.push(CardId::GrandExpectation);
        p.char_states[1].set_hp(6);
    }
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::GrandExpectation, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrailOfTheQilin)),
    ]);
    assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[1].get_hp());
    assert_eq!(7, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
}
