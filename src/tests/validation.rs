use super::*;
use crate::prelude::*;

#[test]
fn switch_character_validation() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
    )
    .ignore_costs(true)
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
fn cast_skill_validation() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
        vector![CharId::Yoimiya, CharId::Xingqiu, CharId::Ganyu],
    )
    .ignore_costs(true)
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
fn cast_skill_cost_validation() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
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
fn weapon_equip_validation() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya, CharId::Keqing], vector![CharId::Fischl])
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.player_mut(PlayerId::PlayerFirst).hand =
        [CardId::SkywardHarp, CardId::SkywardSpine, CardId::SacrificialSword].into();
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
    gs.player_mut(PlayerId::PlayerFirst)
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
fn artifact_equip_validation() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya, CharId::Ganyu], vector![CharId::Fischl])
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand = [CardId::WitchsScorchingHat, CardId::BrokenRimesEcho].into();
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
    gs.player_mut(PlayerId::PlayerFirst)
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
