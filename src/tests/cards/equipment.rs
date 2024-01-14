use super::*;

fn game_state_for_artifacts(card_id: CardId) -> GameState {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::KamisatoAyaka, CharId::Yoimiya],
        vector![CharId::Fischl],
    )
    .enable_log(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.dice.add_in_place(&DiceCounter::omni(8));
    gs.players.0.hand.push(card_id);
    gs.players.0.hand.push(card_id);
    gs
}

#[test]
fn artifact_2_reduces_talent_cost_once_per_round() {
    let mut gs = game_state_for_artifacts(CardId::BrokenRimesEcho);
    assert_eq!(8, gs.players.0.dice.total());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::BrokenRimesEcho, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(6, gs.players.0.dice.total());
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtHyouka),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    // No longer cost reduced
    assert_eq!(4, gs.players.0.dice.total());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KamisatoArtHyouka),
    )]);
    assert_eq!(1, gs.players.0.dice.total());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    // Next round
    gs.advance_roll_phase_no_dice();
    gs.players.0.dice.add_in_place(&DiceCounter::omni(8));
    assert_eq!(9, gs.players.0.dice.total());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
        ),
    ]);
    assert_eq!(7, gs.players.0.dice.total());
}

#[test]
fn artifact_2_does_not_reduce_non_matching_element_cost() {
    let mut gs = game_state_for_artifacts(CardId::BrokenRimesEcho);
    assert_eq!(8, gs.players.0.dice.total());
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::BrokenRimesEcho, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(5, gs.players.0.dice.total());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::NiwabiFireDance),
    )]);
    // Cost not reduced
    assert_eq!(4, gs.players.0.dice.total());
}

#[test]
fn artifact_3_dice_guarantee() {
    let mut gs = game_state_for_artifacts(CardId::BlizzardStrayer);
    assert_eq!(8, gs.players.0.dice.total());
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::BlizzardStrayer, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert_eq!(2, gs.players.0.get_dice_distribution().fixed_count());
    assert_eq!(
        2,
        gs.players.0.get_dice_distribution().fixed_count_for_elem(Element::Cryo)
    );

    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);

    assert_eq!(2, gs.players.0.get_dice_distribution().fixed_count());
    assert_eq!(
        2,
        gs.players.0.get_dice_distribution().fixed_count_for_elem(Element::Cryo)
    );
}

#[test]
fn talent_equip_must_be_on_matching_and_active_character() {
    let mut gs =
        GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Xingqiu, CharId::Yoimiya], vector![CharId::Fischl])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::NaganoharaMeteorSwarm);
    gs.players.0.hand.push(CardId::TheScentRemained);
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::NaganoharaMeteorSwarm, Some(CardSelection::OwnCharacter(1)))
        ))
        .is_err());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::NaganoharaMeteorSwarm, Some(CardSelection::OwnCharacter(0)))
        ))
        .is_err());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::TheScentRemained, Some(CardSelection::OwnCharacter(1)))
        ))
        .is_err());
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::TheScentRemained, Some(CardSelection::OwnCharacter(0)))
        ))
        .is_ok());
}

#[test]
fn talent_equip_without_skill_must_be_on_matching_character() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Xingqiu, CharId::KamisatoAyaka],
        vector![CharId::Fischl],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::KantenSenmyouBlessing);
    assert!(gs
        .clone()
        .advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::KantenSenmyouBlessing, Some(CardSelection::OwnCharacter(1)))
        ))
        .is_ok());
}

#[test]
fn gamblers_earrings_triggers_on_skill_defeat() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Fischl, CharId::Kaeya],
        vector![CharId::Ganyu, CharId::Xingqiu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.1.char_states[0].set_hp(1);
    gs.players.0.hand.push(CardId::GamblersEarrings);
    assert_eq!(0, gs.players.0.dice[Dice::Omni]);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::GamblersEarrings, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(1)),
    ]);
    assert_eq!(2, gs.players.0.dice[Dice::Omni]);
}

#[test]
fn gamblers_earrings_triggers_on_summon_defeat() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Fischl, CharId::Kaeya],
        vector![CharId::Ganyu, CharId::Xingqiu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.1.char_states[0].set_hp(2);
    gs.players.0.hand.push(CardId::GamblersEarrings);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::GamblersEarrings, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert_eq!(1, gs.players.1.char_states[0].get_hp());
    assert_eq!(0, gs.players.0.dice[Dice::Omni]);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(1)),
    ]);
    assert_eq!(2, gs.players.0.dice[Dice::Omni]);
}

#[test]
fn gamblers_earrings_does_not_trigger_on_non_active_defeat() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Fischl, CharId::Kaeya],
        vector![CharId::Ganyu, CharId::Xingqiu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.1.char_states[0].set_hp(2);
    gs.players.0.hand.push(CardId::GamblersEarrings);
    assert_eq!(0, gs.players.0.dice[Dice::Omni]);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::GamblersEarrings, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::CeremonialBladework),
        ),
    ]);
    assert_eq!(0, gs.players.0.dice[Dice::Omni]);
}

#[test]
fn gamblers_earrings_does_not_trigger_on_non_active_summon_defeat() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Fischl, CharId::Kaeya],
        vector![CharId::Ganyu, CharId::Xingqiu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.1.char_states[0].set_hp(2);
    gs.players.0.hand.push(CardId::GamblersEarrings);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::GamblersEarrings, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    assert_eq!(1, gs.players.1.char_states[0].get_hp());
    assert_eq!(0, gs.players.0.dice[Dice::Omni]);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(1)),
    ]);
    assert_eq!(0, gs.players.0.dice[Dice::Omni]);
}

#[test]
fn lithic_spear_grants_shield_points() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Xiangling, CharId::Yoimiya, CharId::Xingqiu],
        vector![CharId::Fischl],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.players.0.hand.push(CardId::LithicSpear);
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::LithicSpear, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert!(gs.players.0.status_collection.has_shield_points());
    assert_eq!(
        2,
        gs.players
            .0
            .status_collection
            .get(StatusKey::Equipment(0, EquipSlot::Weapon, StatusId::LithicSpear))
            .unwrap()
            .get_usages()
    );
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::DoughFu)),
        Input::FromPlayer(
            PlayerId::PlayerSecond,
            PlayerAction::CastSkill(SkillId::BoltsOfDownfall),
        ),
    ]);
    assert!(!gs.players.0.status_collection.has_shield_points());
    assert_eq!(
        0,
        gs.players
            .0
            .status_collection
            .get(StatusKey::Equipment(0, EquipSlot::Weapon, StatusId::LithicSpear))
            .unwrap()
            .get_usages()
    );
}

#[cfg(test)]
mod lucky_dogs_silver_circlet {
    use super::*;

    fn get_game_state() -> GameState {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Xiangling, CharId::Yoimiya, CharId::Xingqiu],
            vector![CharId::Fischl],
        )
        .enable_log(true)
        .ignore_costs(true)
        .build();
        gs.advance_roll_phase_no_dice();
        {
            let p = gs.players.get_mut(PlayerId::PlayerFirst);
            p.hand.push(CardId::LuckyDogsSilverCirclet);
            p.char_states[0].set_hp(5);
        }
        gs.advance_multiple([Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::LuckyDogsSilverCirclet, Some(CardSelection::OwnCharacter(0))),
        )]);
        gs
    }

    #[test]
    fn test_does_not_proc_on_na() {
        let mut gs = get_game_state();
        gs.advance_multiple([Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::DoughFu),
        )]);
        assert_eq!(5, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    }

    #[test]
    fn test_does_not_proc_on_burst() {
        let mut gs = get_game_state();
        gs.advance_multiple([Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::Pyronado),
        )]);
        assert_eq!(5, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    }

    #[test]
    fn test_procs_once_per_round_on_own_skill() {
        let mut gs = get_game_state();
        gs.advance_multiple([Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::GuobaAttack),
        )]);
        assert_eq!(7, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuobaAttack)),
        ]);
        assert_eq!(7, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
            Input::NoAction,
        ]);
        gs.advance_roll_phase_no_dice();
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuobaAttack)),
        ]);
        assert_eq!(9, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    }

    #[test]
    fn test_does_not_proc_on_other_own_character_skill() {
        let mut gs = get_game_state();
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        ]);
        assert_eq!(5, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    }

    #[test]
    fn test_does_not_proc_on_opponent_skill() {
        let mut gs = get_game_state();
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
        ]);
        assert_eq!(4, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    }
}

#[cfg(test)]
mod ornate_kabuto {
    use super::*;

    #[test]
    fn test_does_not_proc_on_own_burst() {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Xingqiu, CharId::Yoimiya],
            vector![CharId::Fischl],
        )
        .enable_log(true)
        .ignore_costs(true)
        .build();
        gs.advance_roll_phase_no_dice();
        gs.players.0.hand.push(CardId::OrnateKabuto);
        gs.advance_multiple([
            Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::PlayCard(CardId::OrnateKabuto, Some(CardSelection::OwnCharacter(0))),
            ),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Raincutter)),
        ]);
        assert_eq!(0, gs.players.0.char_states[0].get_energy());
    }

    #[test]
    fn test_increases_energy_on_teammate_burst() {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Xingqiu, CharId::Yoimiya],
            vector![CharId::Fischl],
        )
        .enable_log(true)
        .ignore_costs(true)
        .build();
        gs.advance_roll_phase_no_dice();
        gs.players.0.hand.push(CardId::OrnateKabuto);
        gs.advance_multiple([
            Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::PlayCard(CardId::OrnateKabuto, Some(CardSelection::OwnCharacter(1))),
            ),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Raincutter)),
        ]);
        assert_eq!(0, gs.players.0.char_states[0].get_energy());
        assert_eq!(1, gs.players.0.char_states[1].get_energy());
    }
}

#[cfg(test)]
mod favonius_sword {
    use super::*;

    #[test]
    fn test_does_not_proc_on_normal_attack() {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Xingqiu, CharId::Yoimiya],
            vector![CharId::Fischl],
        )
        .enable_log(true)
        .ignore_costs(true)
        .build();
        gs.advance_roll_phase_no_dice();
        gs.players.0.hand.push(CardId::FavoniusSword);
        gs.advance_multiple([
            Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::PlayCard(CardId::FavoniusSword, Some(CardSelection::OwnCharacter(0))),
            ),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuhuaStyle)),
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        ]);
        assert_eq!(1, gs.players.0.char_states[0].get_energy());
        assert_eq!(7, gs.players.1.char_states[0].get_hp());
    }

    #[test]
    fn test_adds_energy_after_casting_skill() {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Xingqiu, CharId::Yoimiya],
            vector![CharId::Fischl],
        )
        .enable_log(true)
        .ignore_costs(true)
        .build();
        gs.advance_roll_phase_no_dice();
        gs.players.0.hand.push(CardId::FavoniusSword);
        gs.advance_multiple([
            Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::PlayCard(CardId::FavoniusSword, Some(CardSelection::OwnCharacter(0))),
            ),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FatalRainscreen)),
        ]);
        assert_eq!(2, gs.players.0.char_states[0].get_energy());
    }

    #[test]
    fn test_does_not_proc_with_non_active_character_skill() {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Yoimiya, CharId::Xingqiu],
            vector![CharId::Fischl],
        )
        .enable_log(true)
        .ignore_costs(true)
        .build();
        gs.advance_roll_phase_no_dice();
        gs.players.0.hand.push(CardId::FavoniusSword);
        gs.advance_multiple([
            Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::PlayCard(CardId::FavoniusSword, Some(CardSelection::OwnCharacter(1))),
            ),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        ]);
        assert_eq!(0, gs.players.0.char_states[1].get_energy());
    }
}

mod aquila_favonia {
    use super::*;

    fn init_game_state() -> GameState {
        let mut gs = GameStateBuilder::new_skip_to_roll_phase(
            vector![CharId::Bennett, CharId::Fischl],
            vector![CharId::Yoimiya, CharId::Ganyu],
        )
        .enable_log(true)
        .ignore_costs(true)
        .build();
        gs.advance_roll_phase_no_dice();
        {
            let p = gs.get_player_mut(PlayerId::PlayerFirst);
            p.hand.push(CardId::AquilaFavonia);
            p.char_states[0].set_hp(8);
        }
        gs.advance(Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::AquilaFavonia, Some(CardSelection::OwnCharacter(0))),
        ))
        .unwrap();
        gs
    }

    #[test]
    fn test_increases_dmg() {
        let mut gs = init_game_state();
        gs.advance_multiple([Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::StrikeOfFortune),
        )]);
        assert_eq!(7, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    }

    #[test]
    fn test_heals_on_opponent_skill_cast_when_equipped_is_active_character() {
        let mut gs = init_game_state();
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
            Input::FromPlayer(
                PlayerId::PlayerSecond,
                PlayerAction::CastSkill(SkillId::NiwabiFireDance),
            ),
        ]);
        assert_eq!(9, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
        assert_eq!(
            1,
            gs.get_player(PlayerId::PlayerFirst)
                .status_collection
                .find_equipment(0, EquipSlot::Weapon)
                .unwrap()
                .state
                .get_counter()
        );
    }

    #[test]
    fn test_does_not_heal_on_own_skill_cast() {
        let mut gs = init_game_state();
        gs.advance_multiple([Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::PassionOverload),
        )]);
        assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    }

    #[test]
    fn test_does_not_heal_on_teammate_skill_cast() {
        let mut gs = init_game_state();
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        ]);
        assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    }

    #[test]
    fn test_does_not_heal_on_opponent_skill_cast_when_equipped_is_not_active_character() {
        let mut gs = init_game_state();
        gs.advance_multiple([
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
            Input::FromPlayer(
                PlayerId::PlayerSecond,
                PlayerAction::CastSkill(SkillId::NiwabiFireDance),
            ),
        ]);
        assert_eq!(8, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    }
}
