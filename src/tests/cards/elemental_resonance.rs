use super::*;

use crate::types::game_state::GameState;

#[test]
fn elemental_resonance_sprawling_greenery_does_not_increase_non_reaction_dmg() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Fischl], vector![CharId::Yoimiya]).build();

    gs.players.0.dice.add_single(Dice::Omni, 8);
    gs.players.1.dice.add_single(Dice::Omni, 8);
    gs.players
        .0
        .add_to_hand_ignore(CardId::ElementalResonanceSprawlingGreenery);
    gs.advance_roll_phase_no_dice();

    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::ElementalResonanceSprawlingGreenery, None),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    {
        let yoimiya = &gs.players.1.char_states[0];
        assert_eq!(9, yoimiya.hp());
    }
    assert!(gs
        .status_collection(PlayerId::PlayerFirst)
        .has_team_status(StatusId::ElementalResonanceSprawlingGreenery));
}

#[test]
fn elemental_resonance_sprawling_greenery_increases_reaction_dmg() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Fischl], vector![CharId::Yoimiya]).build();

    gs.players.0.dice.add_single(Dice::Omni, 8);
    gs.players.1.dice.add_single(Dice::Omni, 8);
    gs.players
        .0
        .add_to_hand_ignore(CardId::ElementalResonanceSprawlingGreenery);
    {
        let yoimiya = &mut gs.players.1.char_states[0];
        yoimiya.applied.insert(Element::Pyro);
    }
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::ElementalResonanceSprawlingGreenery, None),
    )]);
    assert!(gs
        .status_collection(PlayerId::PlayerFirst)
        .has_team_status(StatusId::ElementalResonanceSprawlingGreenery));

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::Nightrider),
    )]);
    {
        let yoimiya = &gs.players.1.char_states[0];
        assert!(yoimiya.applied.is_empty());
        assert_eq!(5, yoimiya.hp());
    }
}

#[test]
fn elemental_resonance_sprawling_greenery_increases_usages_of_catalyzing_field() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Fischl, CharId::Collei], vector![CharId::Yoimiya])
            .ignore_costs(true)
            .build();
    gs.players
        .0
        .add_to_hand_ignore(CardId::ElementalResonanceSprawlingGreenery);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::TrumpCardKitty)),
    ]);
    assert_eq!(
        2,
        gs.status_collection(PlayerId::PlayerFirst)
            .get(StatusKey::Team(StatusId::CatalyzingField))
            .unwrap()
            .usages()
    );
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::ElementalResonanceSprawlingGreenery, None),
    )]);
    assert!(gs
        .status_collection(PlayerId::PlayerFirst)
        .has_team_status(StatusId::ElementalResonanceSprawlingGreenery));
    assert_eq!(
        3,
        gs.status_collection(PlayerId::PlayerFirst)
            .get(StatusKey::Team(StatusId::CatalyzingField))
            .unwrap()
            .usages()
    );
    // Cannot create Usages out of nothing
    assert!(!gs
        .status_collection(PlayerId::PlayerFirst)
        .has_team_status(StatusId::DendroCore));
    assert!(!gs
        .status_collection(PlayerId::PlayerFirst)
        .has_summon(SummonId::BurningFlame));
}
