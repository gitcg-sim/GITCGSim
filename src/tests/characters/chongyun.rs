use super::*;

#[test]
fn chonghuas_frost_field_infusion_applies_to_swords() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Chongyun, CharId::Xingqiu],
        vector![CharId::Fischl, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ChonghuasLayeredFrost),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuhuaStyle)),
    ]);
    assert!(gs.players.1.char_states[1].applied.contains(Element::Cryo));
}

#[test]
fn chonghuas_frost_field_infusion_applies_to_polarms() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Chongyun, CharId::Xiangling],
        vector![CharId::Fischl, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ChonghuasLayeredFrost),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::DoughFu)),
    ]);
    assert!(gs.players.1.char_states[1].applied.contains(Element::Cryo));
}

#[test]
fn chonghuas_frost_field_infusion_applies_to_claymores() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Chongyun, CharId::Noelle],
        vector![CharId::Fischl, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ChonghuasLayeredFrost),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ),
    ]);
    assert!(gs.players.1.char_states[1].applied.contains(Element::Cryo));
    assert_eq!(8, gs.players.1.char_states[1].hp());
}

#[test]
fn chonghuas_frost_field_infusion_does_not_apply_to_bows() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Chongyun, CharId::Yoimiya],
        vector![CharId::Fischl, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ChonghuasLayeredFrost),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    assert!(!gs.players.1.char_states[1].applied.contains(Element::Cryo));
}

#[test]
fn chonghuas_frost_field_infusion_does_not_apply_to_catalysts() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Chongyun, CharId::Ningguang],
        vector![CharId::Fischl, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ChonghuasLayeredFrost),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::SparklingScatter),
        ),
    ]);
    assert!(!gs.players.1.char_states[1].applied.contains(Element::Cryo));
}

#[test]
fn chonghuas_frost_field_infusion_does_not_apply_to_others() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Chongyun, CharId::FatuiPyroAgent],
        vector![CharId::Fischl, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ChonghuasLayeredFrost),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Thrust)),
    ]);
    assert!(!gs.players.1.char_states[1].applied.contains(Element::Cryo));
}

#[test]
fn talent_card_affects_chonghuas_frost_field() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Chongyun, CharId::Noelle],
        vector![CharId::Fischl, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.add_to_hand_ignore(CardId::SteadyBreathing);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::SteadyBreathing, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ),
    ]);
    assert_eq!(
        3,
        gs.status_collection(PlayerId::PlayerFirst)
            .get(StatusKey::Team(StatusId::ChonghuaFrostField))
            .unwrap()
            .duration()
    );
    assert!(gs.players.1.char_states[1].applied.contains(Element::Cryo));
    assert_eq!(7, gs.players.1.char_states[1].hp());
}

#[test]
fn talent_card_on_different_character_doesnt_affect_chonghuas_frost_field() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Noelle, CharId::Chongyun],
        vector![CharId::Fischl, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.add_to_hand_ignore(CardId::IGotYourBack);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::IGotYourBack, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(0)),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::ChonghuasLayeredFrost),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(0)),
    ]);
    assert_eq!(10, gs.players.1.char_states[1].hp());
    assert_eq!(elem_set![], gs.players.1.char_states[1].applied);

    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
        ),
    ]);
    assert_eq!(
        2,
        gs.status_collection(PlayerId::PlayerFirst)
            .get(StatusKey::Team(StatusId::ChonghuaFrostField))
            .unwrap()
            .duration()
    );
    assert_eq!(8, gs.players.1.char_states[1].hp());
    assert_eq!(elem_set![Element::Cryo], gs.players.1.char_states[1].applied);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Demonbane)),
    ]);
    assert_eq!(6, gs.players.1.char_states[1].hp());
}
