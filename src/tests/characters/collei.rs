use super::*;

#[test]
fn talent_card() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Collei, CharId::Fischl], vector![CharId::Ganyu])
            .enable_log(true)
            .ignore_costs(true)
            .build();

    gs.players.0.hand.push(CardId::FloralSidewinder);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::FloralSidewinder, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert!(gs.players.0.has_team_status(StatusId::Sprout));
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    {
        let ganyu = gs.players.1.get_active_character();
        assert_eq!(elem_set![Element::Dendro], ganyu.applied);
        assert_eq!(4, ganyu.get_hp());
    }
    assert!(gs.players.0.has_team_status(StatusId::CatalyzingField));
    assert!(!gs.players.0.has_team_status(StatusId::Sprout));
}

#[test]
fn talent_card_immediate_reaction() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Collei, CharId::Fischl], vector![CharId::Ganyu])
            .enable_log(true)
            .ignore_costs(true)
            .build();

    gs.players.0.hand.push(CardId::FloralSidewinder);
    gs.players.1.char_states[0].applied |= Element::Electro;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::FloralSidewinder, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert!(!gs.players.0.has_team_status(StatusId::Sprout));
    {
        let ganyu = gs.players.1.get_active_character();
        assert_eq!(elem_set![Element::Dendro], ganyu.applied);
        assert_eq!(5, ganyu.get_hp());
    }
    assert!(gs.players.0.has_team_status(StatusId::CatalyzingField));
}

#[test]
fn talent_card_does_not_trigger_on_incoming_dendro_reaction() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Collei, CharId::Fischl], vector![CharId::Collei])
            .enable_log(true)
            .ignore_costs(true)
            .build();

    gs.players.0.hand.push(CardId::FloralSidewinder);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::FloralSidewinder, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert!(gs.players.get(PlayerId::PlayerFirst).has_team_status(StatusId::Sprout));
    gs.players.get_mut(PlayerId::PlayerFirst).char_states[0].applied |= Element::Electro;
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::FloralBrush),
    )]);
    assert!(gs.players.get(PlayerId::PlayerFirst).has_team_status(StatusId::Sprout));
    assert!(gs
        .players
        .get(PlayerId::PlayerSecond)
        .has_team_status(StatusId::CatalyzingField));
}
