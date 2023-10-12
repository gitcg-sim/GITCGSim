use super::*;

#[test]
fn test_playing_up_to_4_supports() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::Fischl], vector![CharId::Yoimiya])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::Paimon);
    gs.players.0.hand.push(CardId::Katheryne);
    gs.players.0.hand.push(CardId::Paimon);
    gs.players.0.hand.push(CardId::DawnWinery);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Paimon, None)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Paimon, None)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::DawnWinery, None)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Katheryne, None)),
    ]);
    assert_eq!(
        vec![
            Some(SupportId::Paimon),
            Some(SupportId::Paimon),
            Some(SupportId::DawnWinery),
            Some(SupportId::Katheryne)
        ],
        SupportSlot::VALUES
            .iter()
            .map(|&slot| gs
                .players
                .0
                .status_collection
                .find_support(slot)
                .and_then(|s| s.support_id()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_paimon_adds_omni_dice() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::Fischl], vector![CharId::Yoimiya])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = false;
    gs.players.0.dice.add_in_place(&DiceCounter::omni(3));
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::Paimon);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Paimon, None)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);

    gs.advance_roll_phase_no_dice();
    assert_eq!(2, gs.round_number);
    assert_eq!(2, gs.players.0.dice[Dice::Omni]);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);

    gs.advance_roll_phase_no_dice();
    assert_eq!(3, gs.round_number);
    assert_eq!(4, gs.players.0.dice[Dice::Omni]);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);

    gs.advance_roll_phase_no_dice();
    assert_eq!(4, gs.round_number);
    assert_eq!(4, gs.players.0.dice[Dice::Omni]);
}

#[test]
fn test_multiple_paimon_adds_additional_omni_dice() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::Fischl], vector![CharId::Yoimiya])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = false;
    gs.players.0.dice.add_in_place(&DiceCounter::omni(6));
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::Paimon);
    gs.players.0.hand.push(CardId::Paimon);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Paimon, None)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Paimon, None)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);

    gs.advance_roll_phase_no_dice();
    assert_eq!(2, gs.round_number);
    assert_eq!(4, gs.players.0.dice[Dice::Omni]);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);
}

#[test]
fn test_jade_chamber_guarantees_dice_with_active_character_elem() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::Fischl, CharId::Ganyu], vector![CharId::Yoimiya])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = false;
    gs.advance_roll_phase_no_dice();
    gs.players.0.dice.add_in_place(&DiceCounter::omni(4));
    gs.players.0.hand.push(CardId::JadeChamber);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::JadeChamber, None),
    )]);
    assert!(gs
        .players
        .0
        .status_collection
        .find_support(SupportSlot::Slot0)
        .is_some());
    assert_eq!(2, gs.players.0.get_dice_distribution().fixed_count());
    assert_eq!(
        2,
        gs.players
            .0
            .get_dice_distribution()
            .fixed_count_for_elem(Element::Electro)
    );

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::SwitchCharacter(1),
    )]);
    assert_eq!(2, gs.players.0.get_dice_distribution().fixed_count());
    assert_eq!(
        2,
        gs.players.0.get_dice_distribution().fixed_count_for_elem(Element::Cryo)
    );
}

#[test]
fn test_knights_of_favonius_library_updates_reroll_counts() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::Fischl, CharId::Ganyu], vector![CharId::Yoimiya])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = false;
    gs.advance_roll_phase_no_dice();
    gs.players.0.dice.add_in_place(&DiceCounter::omni(4));
    gs.players.0.hand.push(CardId::KnightsOfFavoniusLibrary);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::KnightsOfFavoniusLibrary, None),
    )]);
    assert!(gs
        .players
        .0
        .status_collection
        .find_support(SupportSlot::Slot0)
        .is_some());
    assert_eq!(2, gs.players.0.get_dice_distribution().rerolls);
}

#[test]
fn test_liben() {
    let mut gs = {
        GameStateBuilder::new_roll_phase_1(vector![CharId::Fischl, CharId::Ganyu], vector![CharId::Yoimiya])
            .with_enable_log(true)
            .build()
    };
    gs.ignore_costs = false;
    gs.advance_roll_phase_no_dice();
    gs.players
        .0
        .dice
        .add_in_place(&DiceCounter::new(&vec![(Dice::Omni, 1), (Dice::CRYO, 2)]));
    gs.players.0.hand.push(CardId::Liben);
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Liben, None)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert_eq!(
        2,
        gs.players
            .0
            .status_collection
            .find_support(SupportSlot::Slot0)
            .unwrap()
            .state
            .get_counter()
    );
    assert_eq!(1, gs.players.0.dice.total());
    assert_eq!(1, gs.players.0.dice[Dice::CRYO]);
    assert_eq!(2, gs.round_number);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
        Input::NondetResult(NondetResult::ProvideCards(
            list8![CardId::BlankCard, CardId::BlankCard],
            list8![],
        )),
    ]);
    assert!(gs
        .players
        .0
        .status_collection
        .find_support(SupportSlot::Slot0)
        .is_none());
    assert_eq!(2, gs.players.0.hand.len());
    assert_eq!(2, gs.players.0.dice.total());
    assert_eq!(2, gs.players.0.dice[Dice::Omni]);
    assert_eq!(3, gs.round_number);
}
