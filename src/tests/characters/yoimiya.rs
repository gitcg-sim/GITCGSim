use super::*;

#[test]
fn niwabi_fire_dance_status() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Yoimiya], vector![CharId::Ganyu])
            .ignore_costs(true)
            .build();

    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::NiwabiFireDance),
    )]);
    {
        assert_eq!(0, gs.active_character().unwrap().energy());
        assert!(gs
            .status_collection(PlayerId::PlayerFirst)
            .get(StatusKey::Character(0, StatusId::NiwabiEnshou))
            .is_some());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);

    {
        let ganyu = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(elem_set![Element::Pyro], ganyu.applied);
        assert_eq!(7, ganyu.hp());
    }
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FireworkFlareUp),
    )]);
    {
        let ganyu = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(4, ganyu.hp());
    }
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FireworkFlareUp),
    )]);
    assert!(gs
        .status_collection_mut(PlayerId::PlayerFirst)
        .get(StatusKey::Character(0, StatusId::NiwabiEnshou))
        .is_none());
    {
        let ganyu = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(2, ganyu.hp());
    }
}

#[test]
fn ryuukin_saxifrage_trigger_duration() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Fischl],
        vector![CharId::Ganyu, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::RyuukinSaxifrage),
    )]);
    assert!(gs
        .status_collection_mut(PlayerId::PlayerFirst)
        .get(StatusKey::Team(StatusId::AurousBlaze))
        .is_some());
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    {
        let kaeya = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(elem_set![], kaeya.applied);
        assert_eq!(8, kaeya.hp());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
    ]);
    {
        let kaeya = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(elem_set![Element::Pyro], kaeya.applied);
        assert_eq!(5, kaeya.hp());
    }
}

#[test]
fn talent_card_costs_niwabi_enshou_and_increases_dmg() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::Fischl],
        vector![CharId::Ganyu, CharId::Kaeya],
    )
    .ignore_costs(true)
    .build();
    gs.players.0.add_to_hand_ignore(CardId::NaganoharaMeteorSwarm);
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::NaganoharaMeteorSwarm, Some(CardSelection::OwnCharacter(0))),
    )]);
    {
        assert!(gs
            .status_collection(PlayerId::PlayerFirst)
            .get(StatusKey::Character(0, StatusId::NiwabiEnshou))
            .is_some());
        let yoimiya = gs.player(PlayerId::PlayerFirst).try_get_character(0).unwrap();
        assert_eq!(0, yoimiya.energy());
        assert!(yoimiya.has_talent_equipped());
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    {
        let ganyu = gs.player(PlayerId::PlayerSecond).active_character();
        assert_eq!(elem_set![Element::Pyro], ganyu.applied);
        assert_eq!(6, ganyu.hp());
    }
}
