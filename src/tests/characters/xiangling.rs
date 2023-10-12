use super::*;

#[test]
fn test_guoba_attack_deals_dmg_at_end_phase() {
    let mut gs =
        GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Xiangling, CharId::Fischl], vector![CharId::Kaeya])
            .with_enable_log(true)
            .with_ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::GuobaAttack)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert!(gs.get_player(PlayerId::PlayerFirst).has_summon(SummonId::Guoba));
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        elem_set![Element::Pyro],
        gs.get_player(PlayerId::PlayerSecond).char_states[0].applied
    );
}

#[test]
fn test_talent_card_deals_pyro_dmg_on_skill_cast() {
    let mut gs =
        GameStateBuilder::new_skip_to_roll_phase(vector![CharId::Xiangling, CharId::Fischl], vector![CharId::Kaeya])
            .with_enable_log(true)
            .with_ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerFirst).hand.push(CardId::Crossfire);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::Crossfire, Some(CardSelection::OwnCharacter(0))),
    )]);
    assert!(gs.get_player(PlayerId::PlayerFirst).has_summon(SummonId::Guoba));
    assert_eq!(9, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        elem_set![Element::Pyro],
        gs.get_player(PlayerId::PlayerSecond).char_states[0].applied
    );
}

#[test]
fn test_pyronado_deals_dmg_on_skill_cast() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Xiangling, CharId::Fischl],
        vector![CharId::Kaeya, CharId::Xingqiu],
    )
    .with_enable_log(true)
    .with_ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Pyronado)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
    ]);
    assert!(gs.get_player(PlayerId::PlayerFirst).has_team_status(StatusId::Pyronado));
    assert_eq!(8, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
    assert_eq!(
        elem_set![Element::Pyro],
        gs.get_player(PlayerId::PlayerSecond).char_states[0].applied
    );
    {
        let mut gs = gs.clone();
        gs.advance_multiple(&vec![Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::DoughFu),
        )]);
        let xingqiu = &gs.get_player(PlayerId::PlayerSecond).char_states[1];
        assert_eq!(6, xingqiu.get_hp());
        assert_eq!(elem_set![Element::Pyro], xingqiu.applied);
    }
    {
        let mut gs = gs.clone();
        gs.advance_multiple(&vec![Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::GuobaAttack),
        )]);
        let xingqiu = &gs.get_player(PlayerId::PlayerSecond).char_states[1];
        assert_eq!(8, xingqiu.get_hp());
        assert_eq!(elem_set![Element::Pyro], xingqiu.applied);
    }
    {
        let mut gs = gs.clone();
        gs.advance_multiple(&vec![
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        ]);
        let xingqiu = &gs.get_player(PlayerId::PlayerSecond).char_states[1];
        assert_eq!(6, xingqiu.get_hp());
        assert_eq!(elem_set![Element::Pyro], xingqiu.applied);
    }
    {
        let mut gs = gs.clone();
        gs.advance_multiple(&vec![
            Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
            Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::GuhuaStyle)),
        ]);
        let xingqiu = &gs.get_player(PlayerId::PlayerSecond).char_states[1];
        assert_eq!(10, xingqiu.get_hp());
        assert_eq!(elem_set![], xingqiu.applied);
    }
}
