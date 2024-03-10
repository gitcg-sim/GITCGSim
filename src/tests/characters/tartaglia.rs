use super::*;

#[test]
fn foul_legacy_raging_tide_melee_stance_and_riptide_transfer() {
    let mut gs: GameState<()> = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Tartaglia],
        vector![CharId::Ganyu, CharId::Fischl],
    )
    .build();

    gs.advance_roll_phase_no_dice();
    gs.player_mut(PlayerId::PlayerFirst).dice.add_single(Dice::Omni, 9);
    {
        let p2 = gs.player_mut(PlayerId::PlayerSecond);
        p2.dice.add_single(Dice::Omni, 1);
        p2.char_states[1].set_hp(3);
    }
    {
        assert!(gs.has_active_character_status(PlayerId::PlayerFirst, StatusId::RangedStance));
        assert!(!gs.has_active_character_status(PlayerId::PlayerFirst, StatusId::MeleeStance));
    }
    gs.advance_multiple([
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::FoulLegacyRagingTide),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(1)),
    ]);
    {
        assert!(!gs.has_active_character_status(PlayerId::PlayerFirst, StatusId::RangedStance));
        assert!(gs.has_active_character_status(PlayerId::PlayerFirst, StatusId::MeleeStance));
    }

    assert!(gs
        .player(PlayerId::PlayerFirst)
        .flags
        .contains(PlayerFlag::ChargedAttack));
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::CuttingTorrent)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert!(gs.has_character_status(PlayerId::PlayerSecond, 1, StatusId::Riptide));
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::CuttingTorrent)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PostDeathSwitch(0)),
    ]);
    assert!(gs.has_character_status(PlayerId::PlayerSecond, 0, StatusId::Riptide));
}
