use super::*;

fn set_stacks(gs: &mut GameState, n: u8) {
    gs.get_player_mut(PlayerId::PlayerFirst)
        .status_collection
        .get_mut(StatusKey::Character(0, StatusId::RadicalVitality))
        .unwrap()
        .set_counter(n)
}

fn get_stacks(gs: &GameState) -> u8 {
    gs.get_player(PlayerId::PlayerFirst)
        .status_collection
        .get(StatusKey::Character(0, StatusId::RadicalVitality))
        .unwrap()
        .get_counter()
}

#[test]
pub fn test_3_radical_vitality_stacks_clear_on_end_phase() {
    let mut gs =
        GameStateBuilder::new_skip_to_roll_phase(vector![CharId::JadeplumeTerrorshroom], vector![CharId::Ganyu])
            .enable_log(true)
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    set_stacks(&mut gs, 3);

    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);

    assert_eq!(0, get_stacks(&gs));
}

#[test]
pub fn test_radival_vitality_stacks_increases_on_own_elemental_dmg_dealt() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::JadeplumeTerrorshroom, CharId::Kaeya],
        vector![CharId::Ganyu],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    assert_eq!(0, get_stacks(&gs));

    // Physical DMG
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::MajesticDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
    ]);
    assert_eq!(0, get_stacks(&gs));
    // Elemental DMG
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::VolatileSporeCloud),
    )]);
    assert_eq!(1, get_stacks(&gs));
    // Other character dealt Elemental DMG
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::Frostgnaw)),
    ]);
    assert_eq!(1, get_stacks(&gs));
}

#[test]
pub fn test_radival_vitality_stacks_increases_on_own_elemental_dmg_received() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::JadeplumeTerrorshroom, CharId::Kaeya],
        vector![CharId::Noelle],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    assert_eq!(0, get_stacks(&gs));

    // Received Geo DMG
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::EndRound),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Breastplate)),
    ]);
    // Received Physical DMG
    assert_eq!(1, get_stacks(&gs));
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::FavoniusBladeworkMaid),
    )]);
    assert_eq!(1, get_stacks(&gs));
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::EndRound),
        Input::NoAction,
    ]);
    assert_eq!(7, gs.get_player(PlayerId::PlayerFirst).char_states[0].get_hp());
    gs.advance_roll_phase_no_dice();
    assert_eq!(1, get_stacks(&gs));
    // Other received Geo DMG
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Breastplate)),
    ]);
    assert_eq!(1, get_stacks(&gs));
}

#[test]
pub fn test_feather_spreading_consumes_radical_vitality_stacks() {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::JadeplumeTerrorshroom, CharId::Kaeya],
        vector![CharId::Noelle],
    )
    .enable_log(true)
    .ignore_costs(true)
    .build();
    gs.advance_roll_phase_no_dice();
    set_stacks(&mut gs, 2);

    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::FeatherSpreading),
    )]);
    assert_eq!(0, get_stacks(&gs));
    assert_eq!(4, gs.get_player(PlayerId::PlayerSecond).char_states[0].get_hp());
}
