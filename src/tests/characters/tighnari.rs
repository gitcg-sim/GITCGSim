use super::*;

#[test]
fn vijnana_phala_mine_charged_attack() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Tighnari], vector![CharId::Fischl])
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
    assert!(gs.has_active_character_status(PlayerId::PlayerFirst, StatusId::VijnanaSuffusion));
    {
        let fischl = &mut gs.player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(8, fischl.hp());
        assert_eq!(elem_set![Element::Dendro], fischl.applied);
        fischl.applied.clear();
    }
    assert!(gs
        .player(PlayerId::PlayerFirst)
        .flags
        .contains(PlayerFlag::ChargedAttack));

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KhandaBarrierBuster),
    )]);
    {
        let fischl = &mut gs.player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(6, fischl.hp());
        assert_eq!(elem_set![Element::Dendro], fischl.applied);
    }
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::ClusterbloomArrow));
}

#[test]
fn vijnana_phala_mine_non_charged_attack() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Tighnari], vector![CharId::Fischl])
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.players.get_mut(PlayerId::PlayerFirst).dice.add_single(Dice::Omni, 1);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::VijnanaPhalaMine),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs.has_active_character_status(PlayerId::PlayerFirst, StatusId::VijnanaSuffusion));
    {
        let fischl = &mut gs.player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(8, fischl.hp());
        assert_eq!(elem_set![Element::Dendro], fischl.applied);
        fischl.applied.clear();
    }
    assert!(!gs
        .player(PlayerId::PlayerFirst)
        .flags
        .contains(PlayerFlag::ChargedAttack));

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KhandaBarrierBuster),
    )]);
    {
        let fischl = &mut gs.player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(6, fischl.hp());
        assert_eq!(elem_set![], fischl.applied);
    }
    assert!(!gs.has_summon(PlayerId::PlayerFirst, SummonId::ClusterbloomArrow));
}

#[test]
fn talent_card_charged_attack() {
    let mut gs: GameState<()> =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Tighnari], vector![CharId::Fischl]).build();

    gs.advance_roll_phase_no_dice();
    {
        let dice = &mut gs.players.get_mut(PlayerId::PlayerFirst).dice;
        dice.add_tally([(Dice::DENDRO, 5), (Dice::PYRO, 1)]);
    }
    gs.players
        .get_mut(PlayerId::PlayerFirst)
        .add_to_hand_ignore(CardId::KeenSight);
    dbg!(&gs.player(PlayerId::PlayerFirst).dice);
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::PlayCard(CardId::KeenSight, Some(CardSelection::OwnCharacter(0))),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs.has_active_character_status(PlayerId::PlayerFirst, StatusId::VijnanaSuffusion));
    assert!(gs
        .player(PlayerId::PlayerFirst)
        .flags
        .contains(PlayerFlag::ChargedAttack));

    gs.advance_multiple([Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::KhandaBarrierBuster),
    )]);
    {
        let fischl = &mut gs.player_mut(PlayerId::PlayerSecond).char_states[0];
        assert_eq!(6, fischl.hp());
        assert_eq!(elem_set![Element::Dendro], fischl.applied);
    }
    assert!(gs.has_summon(PlayerId::PlayerFirst, SummonId::ClusterbloomArrow));
}
