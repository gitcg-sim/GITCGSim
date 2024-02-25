use super::*;

#[test]
fn vijnana_phala_mine_charged_attack() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Tighnari], vector![CharId::Fischl])
        .enable_log(true)
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::VijnanaPhalaMine),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_active_character_status(StatusId::VijnanaSuffusion));
    {
        let fischl = &mut gs.get_player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(8, fischl.get_hp());
        assert_eq!(elem_set![Element::Dendro], fischl.applied);
        fischl.applied.clear();
    }
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .flags
        .contains(PlayerFlag::ChargedAttack));

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KhandaBarrierBuster),
    )]);
    {
        let fischl = &mut gs.get_player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(6, fischl.get_hp());
        assert_eq!(elem_set![Element::Dendro], fischl.applied);
    }
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_summon(SummonId::ClusterbloomArrow));
}

#[test]
fn vijnana_phala_mine_non_charged_attack() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Tighnari], vector![CharId::Fischl])
        .enable_log(true)
        .ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.players
        .get_mut(PlayerId::PlayerFirst)
        .dice
        .add_in_place(&DiceCounter::omni(1));
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::VijnanaPhalaMine),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_active_character_status(StatusId::VijnanaSuffusion));
    {
        let fischl = &mut gs.get_player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(8, fischl.get_hp());
        assert_eq!(elem_set![Element::Dendro], fischl.applied);
        fischl.applied.clear();
    }
    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .flags
        .contains(PlayerFlag::ChargedAttack));

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KhandaBarrierBuster),
    )]);
    {
        let fischl = &mut gs.get_player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(6, fischl.get_hp());
        assert_eq!(elem_set![], fischl.applied);
    }
    assert!(!gs
        .get_player(PlayerId::PlayerFirst)
        .has_summon(SummonId::ClusterbloomArrow));
}

#[test]
fn talent_card_charged_attack() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Tighnari], vector![CharId::Fischl])
        .enable_log(true)
        .build();

    gs.advance_roll_phase_no_dice();
    {
        let dice = &mut gs.players.get_mut(PlayerId::PlayerFirst).dice;
        dice.add_in_place(&DiceCounter::elem(Element::Dendro, 5));
        dice.add_in_place(&DiceCounter::elem(Element::Pyro, 1));
    }
    gs.players
        .get_mut(PlayerId::PlayerFirst)
        .add_to_hand_ignore(CardId::KeenSight);
    dbg!(&gs.get_player(PlayerId::PlayerFirst).dice);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::KeenSight, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_active_character_status(StatusId::VijnanaSuffusion));
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .flags
        .contains(PlayerFlag::ChargedAttack));

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KhandaBarrierBuster),
    )]);
    {
        let fischl = &mut gs.get_player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(6, fischl.get_hp());
        assert_eq!(elem_set![Element::Dendro], fischl.applied);
    }
    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .has_summon(SummonId::ClusterbloomArrow));
}
