use crate::{
    dispatcher_ops::types::DispatchError,
    types::{dice_counter::DiceCounter, enums::Dice},
};

use super::*;

#[test]
fn test_switch_character_validation() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
    )
    .with_ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();

    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::SwitchCharacter(0)
        ))
        .is_err());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::SwitchCharacter(1)
        ))
        .is_ok());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::SwitchCharacter(2)
        ))
        .is_ok());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::SwitchCharacter(3)
        ))
        .is_err());
    assert_eq!(
        Err(DispatchError::InvalidPlayer),
        gs.clone().advance(Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::SwitchCharacter(1)
        ))
    );
}

#[test]
fn test_cast_skill_validation() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
    )
    .with_ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();

    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::NiwabiFireDance)
        ))
        .is_ok());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall)
        ))
        .is_err());
    assert_eq!(
        Err(DispatchError::InvalidPlayer),
        gs.clone().advance(Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::NiwabiFireDance)
        ))
    );
}

#[test]
fn test_cast_skill_cost_validation() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
    )
    .build();
    gs.advance_roll_phase_no_dice();

    gs.players.0.dice = DiceCounter::new(&vec![]);
    assert_eq!(
        Err(DispatchError::UnableToPayCost),
        gs.clone().advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::NiwabiFireDance)
        ))
    );
    gs.players.0.dice = DiceCounter::new(&vec![(Dice::Omni, 8)]);
    assert_eq!(
        Err(DispatchError::UnableToPayCost),
        gs.clone().advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::RyuukinSaxifrage)
        ))
    );
}

#[test]
fn test_weapon_equip_validation() {
    let mut gs =
        GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Yoimiya, CharId::Keqing], vector![CharId::Fischl])
            .with_ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerFirst).hand =
        vector![CardId::SkywardHarp, CardId::SkywardSpine, CardId::SacrificialSword];
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::SkywardSpine, None)
        ),)
        .is_err());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::SkywardSpine, Some(CardSelection::OwnCharacter(0)))
        ),)
        .is_err());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::SacrificialSword, Some(CardSelection::OwnCharacter(0)))
        ),)
        .is_err());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::SkywardHarp, Some(CardSelection::OwnCharacter(0)))
        ),)
        .is_ok());

    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::SacrificialSword, Some(CardSelection::OwnCharacter(1)))
        ),)
        .is_ok());

    // reduce HP to zero
    gs.get_player_mut(PlayerId::PlayerFirst)
        .try_get_character_mut(1)
        .unwrap()
        .reduce_hp(10);
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::SacrificialSword, Some(CardSelection::OwnCharacter(1)))
        ),)
        .is_err());
}

#[test]
fn test_artifact_equip_validation() {
    let mut gs =
        GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Yoimiya, CharId::Ganyu], vector![CharId::Fischl])
            .with_ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand = vector![CardId::WitchsScorchingHat, CardId::BrokenRimesEcho];
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::BrokenRimesEcho, Some(CardSelection::OwnCharacter(0)))
        ),)
        .is_ok());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::BrokenRimesEcho, Some(CardSelection::OwnCharacter(1)))
        ),)
        .is_ok());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::WitchsScorchingHat, Some(CardSelection::OwnCharacter(0)))
        ),)
        .is_ok());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::WitchsScorchingHat, Some(CardSelection::OwnCharacter(1)))
        ),)
        .is_ok());

    // reduce HP to zero
    gs.get_player_mut(PlayerId::PlayerFirst)
        .try_get_character_mut(1)
        .unwrap()
        .reduce_hp(10);
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::BrokenRimesEcho, Some(CardSelection::OwnCharacter(1)))
        ),)
        .is_err());
}
