use super::*;

#[test]
fn foul_legacy_raging_tide_melee_stance_and_riptide_transfer() {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Tartaglia],
        vector![CharId::Ganyu, CharId::Fischl],
    )
    .enable_log(true)
    .build();

    gs.advance_roll_phase_no_dice();
    gs.get_player_mut(PlayerId::PlayerFirst)
        .dice
        .add_in_place(&DiceCounter::omni(9));
    {
        let p2 = gs.get_player_mut(PlayerId::PlayerSecond);
        p2.dice.add_in_place(&DiceCounter::omni(1));
        p2.char_states[1].set_hp(3);
    }
    {
        let p = gs.get_player(PlayerId::PlayerFirst);
        assert!(p.has_active_character_status(StatusId::RangedStance));
        assert!(!p.has_active_character_status(StatusId::MeleeStance));
    }
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FoulLegacyRagingTide),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
    ]);
    {
        let p = gs.get_player(PlayerId::PlayerFirst);
        assert!(!p.has_active_character_status(StatusId::RangedStance));
        assert!(p.has_active_character_status(StatusId::MeleeStance));
    }

    assert!(gs
        .get_player(PlayerId::PlayerFirst)
        .flags
        .contains(PlayerFlag::ChargedAttack));
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::CuttingTorrent)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    {
        let p2 = gs.get_player(PlayerId::PlayerSecond);
        assert!(p2.has_character_status(1, StatusId::Riptide));
    }
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::CuttingTorrent)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(0)),
    ]);
    {
        let p2 = gs.get_player(PlayerId::PlayerSecond);
        assert!(p2.has_character_status(0, StatusId::Riptide));
    }
}
