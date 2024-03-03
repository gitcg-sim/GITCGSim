use crate::prelude::ByPlayer;

use super::*;

pub mod equipment;

pub mod support;

pub mod elemental_resonance;

#[test]
fn changing_shifts() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Kaeya, CharId::Fischl],
        vector![CharId::KamisatoAyaka],
    )
    .build();

    gs.advance_roll_phase_no_dice();
    assert_eq!(0, gs.players.0.dice.total());
    gs.players.0.add_to_hand_ignore(CardId::ChangingShifts);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::ChangingShifts, None),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
    ]);
    assert!(!gs.has_team_status(PlayerId::PlayerFirst, StatusId::ChangingShifts));
}

#[test]
fn changing_shifts_not_cleared() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Kaeya, CharId::Fischl],
        vector![CharId::KamisatoAyaka],
    )
    .build();

    gs.advance_roll_phase_no_dice();
    assert_eq!(0, gs.players.0.dice.total());
    gs.players.0.add_to_hand_ignore(CardId::ChangingShifts);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::ChangingShifts, None),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    gs.advance_roll_phase_no_dice();
    assert_eq!(2, gs.round_number);
    assert!(gs.has_team_status(PlayerId::PlayerFirst, StatusId::ChangingShifts));
}

#[test]
fn leave_it_to_me() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Kaeya, CharId::Fischl],
        vector![CharId::KamisatoAyaka],
    )
    .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.dice[Dice::Omni] += 2;
    gs.players.0.add_to_hand_ignore(CardId::LeaveItToMe);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::LeaveItToMe, None)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
    ]);
    assert_eq!(Some(PlayerId::PlayerFirst), gs.to_move_player());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::SwitchCharacter(0),
    )]);
    assert!(!gs.has_team_status(PlayerId::PlayerFirst, StatusId::LeaveItToMe));
    assert_eq!(Some(PlayerId::PlayerSecond), gs.to_move_player());
}

#[test]
fn food() {
    // TODO food implementation: target character and once per turn check
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Kaeya, CharId::Fischl, CharId::Yoimiya],
        vector![CharId::KamisatoAyaka],
    )
    .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.char_states[0].reduce_hp(5);
    gs.players.0.char_states[1].reduce_hp(5);
    gs.players.0.char_states[2].reduce_hp(5);
    gs.players.0.add_to_hand_ignore(CardId::SweetMadame);
    gs.players.0.add_to_hand_ignore(CardId::SweetMadame);
    gs.players.0.add_to_hand_ignore(CardId::SweetMadame);
    {
        let mut gs = gs.clone();
        assert_eq!(
            Err(DispatchError::InvalidSelection),
            gs.advance(Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::PlayCard(CardId::SweetMadame, None)
            ))
        )
    }
    gs.advance(Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::SweetMadame, Some(CardSelection::OwnCharacter(2))),
    ))
    .unwrap();
    assert_eq!(6, gs.players.0.char_states[2].get_hp());
    {
        let mut gs = gs.clone();
        assert_eq!(
            Err(DispatchError::InvalidSelection),
            gs.advance(Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::PlayCard(CardId::SweetMadame, Some(CardSelection::OwnCharacter(2)))
            ))
        )
    }
    gs.advance(Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::SweetMadame, Some(CardSelection::OwnCharacter(0))),
    ))
    .unwrap();
    assert_eq!(6, gs.players.0.char_states[0].get_hp());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        NO_ACTION,
    ]);
    gs.advance_roll_phase_no_dice();
    gs.advance(Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::SweetMadame, Some(CardSelection::OwnCharacter(2))),
    ))
    .unwrap();
    assert_eq!(7, gs.players.0.char_states[2].get_hp());
}

#[test]
fn i_havent_lost_yet_activation_condition() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Kaeya, CharId::Fischl], vector![CharId::Yoimiya])
            .build();

    gs.players.0.dice.add_in_place(&DiceCounter::omni(8));
    gs.players.1.dice.add_in_place(&DiceCounter::omni(8));
    gs.players.0.add_to_hand_ignore(CardId::IHaventLostYet);
    gs.players.0.try_get_character_mut(1).unwrap().set_hp(1);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::FireworkFlareUp),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PostDeathSwitch(0)),
    ]);

    assert!(gs.available_actions().contains(&Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::IHaventLostYet, None)
    )));
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        NO_ACTION,
    ]);
    gs.advance_roll_phase_no_dice();
    assert_eq!(2, gs.round_number);
    assert_eq!(
        Phase::ActionPhase {
            first_end_round: None,
            active_player: PlayerId::PlayerFirst
        },
        gs.phase
    );
    assert!(!gs.available_actions().contains(&Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::IHaventLostYet, None)
    )));
}

#[test]
fn strategize() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Kaeya], vector![CharId::Yoimiya]).build();

    gs.players.0.dice.add_in_place(&DiceCounter::omni(8));
    gs.players.1.dice.add_in_place(&DiceCounter::omni(8));
    gs.players.0.add_to_hand_ignore(CardId::IHaventLostYet);
    gs.players.0.add_to_hand_ignore(CardId::Strategize);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::Strategize, None),
    )]);
    gs.advance(Input::NondetResult(NondetResult::ProvideCards(ByPlayer(
        list8![CardId::Paimon],
        list8![],
    ))))
    .unwrap();
    assert_eq!([CardId::IHaventLostYet, CardId::Paimon], gs.players.0.hand.slice());
}

#[test]
fn quick_knit() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Fischl], vector![CharId::Yoimiya])
        .ignore_costs(true)
        .build();
    gs.players.0.add_to_hand_ignore(CardId::QuickKnit);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::QuickKnit, Some(CardSelection::OwnSummon(SummonId::Oz))),
        ),
    ]);
    assert_eq!(
        3,
        gs.get_status_collection(PlayerId::PlayerFirst)
            .get(StatusKey::Summon(SummonId::Oz))
            .unwrap()
            .get_usages()
    )
}

#[test]
fn send_off() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya], vector![CharId::Fischl])
        .ignore_costs(true)
        .build();
    gs.players.0.add_to_hand_ignore(CardId::SendOff);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert!(gs
        .get_status_collection(PlayerId::PlayerSecond)
        .get(StatusKey::Summon(SummonId::Oz))
        .is_some());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::SendOff, Some(CardSelection::OpponentSummon(SummonId::Oz))),
    )]);
    assert!(gs
        .get_status_collection(PlayerId::PlayerSecond)
        .get(StatusKey::Summon(SummonId::Oz))
        .is_none())
}

#[test]
fn calxs_arts() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Ganyu, CharId::Xingqiu],
        vector![CharId::Fischl],
    )
    .build();
    gs.players.0.add_to_hand_ignore(CardId::CalxsArts);
    {
        let char_states = &mut gs.players.0.char_states;
        char_states[0].set_energy(2);
        char_states[1].set_energy(1);
        char_states[2].set_energy(1);
    }
    gs.ignore_costs = true;
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::CalxsArts, None),
    )]);
    {
        let char_states = &mut gs.players.0.char_states;
        assert_eq!(3, char_states[0].get_energy());
        assert_eq!(0, char_states[1].get_energy());
        assert_eq!(0, char_states[2].get_energy());
    }
}
