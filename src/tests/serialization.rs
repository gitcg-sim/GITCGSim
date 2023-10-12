use rand::{rngs::SmallRng, SeedableRng};

use crate::{
    deck::{sample_deck, Decklist},
    game_tree_search::{Game, GameStateWrapper, ZobristHashable},
    types::nondet::{NondetProvider, StandardNondetHandlerState},
};

use super::*;

fn _initial_gs() -> GameState {
    let mut gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::KamisatoAyaka, CharId::Yoimiya, CharId::Collei],
        vector![CharId::Diona, CharId::Ningguang, CharId::Noelle],
    )
    .with_enable_log(false)
    .build();
    gs.advance_multiple(&vec![
        Input::NoAction,
        Input::NondetResult(NondetResult::ProvideCards(
            list8![CardId::Paimon, CardId::BrokenRimesEcho, CardId::Strategize],
            list8![CardId::DawnWinery, CardId::AdeptusTemptation, CardId::Strategize],
        )),
        Input::NondetResult(NondetResult::ProvideDice(
            DiceCounter::omni(8),
            DiceCounter::elem(Element::Cryo, 8),
        )),
        Input::FromPlayer(
            PlayerId::PlayerFirst,
            PlayerAction::CastSkill(SkillId::KamisatoArtKabuki),
        ),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::IcyPaws)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::SwitchCharacter(1)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::SwitchCharacter(2)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::PlayCard(CardId::Strategize, None)),
        Input::NondetResult(NondetResult::ProvideCards(list8![CardId::IHaventLostYet], list8![])),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::PlayCard(CardId::DawnWinery, None)),
    ]);
    gs
}

fn _initial_gs_wrapper() -> GameStateWrapper<StandardNondetHandlerState> {
    let decklist1 = Decklist::new(Default::default(), sample_deck());
    let decklist2 = Decklist::new(Default::default(), sample_deck());
    let gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Klee, CharId::Xingqiu, CharId::KamisatoAyaka],
        vector![CharId::Noelle, CharId::Ningguang, CharId::Fischl],
    )
    .with_enable_log(false)
    .build();
    let state = StandardNondetHandlerState::new(&decklist1, &decklist2, SmallRng::seed_from_u64(100).into());
    let nd = NondetProvider::new(state);
    let mut gsw = GameStateWrapper::new(gs, nd);
    gsw.advance(Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::PlayCard(CardId::Paimon, None),
    ))
    .unwrap();
    gsw.advance(Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::CastSkill(SkillId::JumpyDumpty),
    ))
    .unwrap();
    dbg!(&gsw.game_state);
    gsw.advance(Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::PlayCard(CardId::TheBestestTravelCompanion, None),
    ))
    .unwrap();
    gsw.advance(Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::CastSkill(SkillId::Breastplate),
    ))
    .unwrap();
    gsw.advance(Input::FromPlayer(
        PlayerId::PlayerFirst,
        PlayerAction::SwitchCharacter(1),
    ))
    .unwrap();
    gsw
}

#[test]
fn test_game_state_serialize_json() {
    let gs = _initial_gs();
    let ser = serde_json::to_string_pretty(&gs).unwrap();
    println!("{ser}");
    let mut gs1: GameState = serde_json::from_str(&ser).unwrap();
    gs1.rehash();
    assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
}

#[test]
fn test_game_state_serialize_bincode() {
    let gs = _initial_gs();
    let ser = bincode::serialize(&gs).unwrap();
    println!("{ser:?}");
    let mut gs1: GameState = bincode::deserialize(&ser).unwrap();
    gs1.rehash();
    assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
}

#[test]
fn test_gsw_serialize_json() {
    let gs = _initial_gs_wrapper();
    let ser = serde_json::to_string_pretty(&gs).unwrap();
    println!("{ser}");
    let mut gs1: GameStateWrapper<StandardNondetHandlerState> = serde_json::from_str(&ser).unwrap();
    gs1.game_state.rehash();
    assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
}

#[test]
fn test_gsw_serialize_bincode() {
    let gs = _initial_gs_wrapper();
    let ser = bincode::serialize(&gs).unwrap();
    println!("{ser:?}");
    let mut gs1: GameStateWrapper<StandardNondetHandlerState> = bincode::deserialize(&ser).unwrap();
    gs1.game_state.rehash();
    assert_eq!(gs.zobrist_hash(), gs1.zobrist_hash());
}
