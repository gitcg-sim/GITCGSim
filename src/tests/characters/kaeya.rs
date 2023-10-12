use super::*;

#[test]
fn test_glacial_waltz_switch_trigger() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Kaeya, CharId::Fischl],
        vector![CharId::Xiangling, CharId::Yoimiya],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GlacialWaltz)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
    ]);
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).get_active_character().get_hp());
    assert_eq!(
        elem_set![Element::Cryo],
        gs.get_player(PlayerId::PlayerSecond).get_active_character().applied
    );
}

#[test]
fn test_talent_card() {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Kaeya, CharId::Fischl],
        vector![CharId::Xiangling, CharId::Yoimiya],
    )
    .with_enable_log(true)
    .build();
    gs.ignore_costs = true;
    gs.players.0.hand.push(CardId::ColdBloodedStrike);

    gs.players.0.char_states[0].set_hp(5);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::ColdBloodedStrike, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs
        .players
        .0
        .status_collection
        .get(StatusKey::Equipment(0, EquipSlot::Talent, StatusId::ColdBloodedStrike))
        .is_some());
    assert_eq!(7, gs.players.0.char_states[0].get_hp());
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Frostgnaw),
    )]);
    assert_eq!(7, gs.players.0.char_states[0].get_hp());
}
