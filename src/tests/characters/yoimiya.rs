use super::*;

#[test]
fn niwabi_fire_dance_status() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Yoimiya], vector![CharId::Ganyu])
        .enable_log(true)
        .ignore_costs(true)
        .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::NiwabiFireDance),
    )]);
    {
        let p = gs.get_player(PlayerId::PlayerFirst);
        assert_eq!(0, gs.get_active_character().unwrap().get_energy());
        assert!(p
            .status_collection
            .get(StatusKey::Character(0, StatusId::NiwabiEnshou))
            .is_some());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);

    {
        let ganyu = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Pyro], ganyu.applied);
        assert_eq!(7, ganyu.get_hp());
    }
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FireworkFlareUp),
    )]);
    {
        let ganyu = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(4, ganyu.get_hp());
    }
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FireworkFlareUp),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Character(0, StatusId::NiwabiEnshou))
        .is_none());
    {
        let ganyu = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(2, ganyu.get_hp());
    }
}

#[test]
fn ryuukin_saxifrage_trigger_duration() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Fischl],
        vector![CharId::Ganyu, CharId::Kaeya],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::RyuukinSaxifrage),
    )]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Team(StatusId::AurousBlaze))
        .is_some());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    {
        let kaeya = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![], kaeya.applied);
        assert_eq!(8, kaeya.get_hp());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
    ]);
    #[cfg(feature = "std")]
    gs.log.print();
    {
        let kaeya = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Pyro], kaeya.applied);
        assert_eq!(5, kaeya.get_hp());
    }
}

#[test]
fn talent_card_costs_niwabi_enshou_and_increases_dmg() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Fischl],
        vector![CharId::Ganyu, CharId::Kaeya],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.players.0.hand.push(CardId::NaganoharaMeteorSwarm);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::NaganoharaMeteorSwarm, Some(CardSelection::OwnCharacter(0))),
    )]);
    {
        let p = gs.get_player(PlayerId::PlayerFirst);
        assert!(p
            .status_collection
            .get(StatusKey::Character(0, StatusId::NiwabiEnshou))
            .is_some());
        let yoimiya = p.try_get_character(0).unwrap();
        assert_eq!(0, yoimiya.get_energy());
        assert!(yoimiya.has_talent_equipped());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    {
        let ganyu = gs.get_player(PlayerId::PlayerSecond).get_active_character();
        assert_eq!(elem_set![Element::Pyro], ganyu.applied);
        assert_eq!(6, ganyu.get_hp());
    }
}
