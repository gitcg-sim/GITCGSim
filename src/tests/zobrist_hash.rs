use rand::{rngs::SmallRng, RngCore, SeedableRng};

use crate::{
    deck::{sample_deck, Decklist},
    types::nondet::{NondetProvider, StandardNondetHandlerState},
    zobrist_hash::ZobristHasher,
};

use super::*;

#[test]
fn test_zobrist_hash() {
    let mut gs = GameStateBuilder::new_roll_phase_1(vector![CharId::Fischl], vector![CharId::Kaeya, CharId::Yoimiya])
        .with_enable_log(true)
        .with_ignore_costs(true)
        .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple(&vec![
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::GlacialWaltz)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::BoltsOfDownfall)),
    ]);
    let h = {
        let mut h = ZobristHasher::new();
        gs.zobrist_hash_full_recompute(&mut h);
        h.finish()
    };
    dbg!(h);
    gs.advance_multiple(&vec![Input::FromPlayer(
        PlayerId::PlayerSecond,
        PlayerAction::SwitchCharacter(1),
    )]);
    let h = {
        let mut h = ZobristHasher::new();
        gs.zobrist_hash_full_recompute(&mut h);
        h.finish()
    };
    dbg!(h);
}

#[test]
fn test_zobrist_hash_random_steps_1() {
    let gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Keqing, CharId::Yoimiya, CharId::Cyno],
        vector![CharId::Klee, CharId::Xingqiu, CharId::Mona],
    )
    .with_enable_log(false)
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

#[test]
fn test_zobrist_hash_random_steps_2() {
    let gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::KamisatoAyaka, CharId::Yoimiya, CharId::Collei],
        vector![CharId::Diona, CharId::Ningguang, CharId::Noelle],
    )
    .with_enable_log(false)
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

#[test]
fn test_zobrist_hash_random_steps_3() {
    let gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Barbara, CharId::Fischl, CharId::Collei],
        vector![CharId::Mona, CharId::Ganyu, CharId::Kaeya],
    )
    .with_enable_log(false)
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

#[test]
fn test_zobrist_hash_random_steps_4() {
    let gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::KujouSara, CharId::Keqing, CharId::SangonomiyaKokomi],
        vector![CharId::Mona, CharId::Cyno, CharId::Eula],
    )
    .with_enable_log(false)
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

#[test]
fn test_zobrist_hash_random_steps_5() {
    let gs = GameStateBuilder::new_roll_phase_1(
        vector![CharId::Eula, CharId::Fischl, CharId::Diona],
        vector![CharId::KamisatoAyaka, CharId::Ganyu, CharId::Xingqiu],
    )
    .with_enable_log(false)
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

fn _test_zobrist_hash_random_steps(mut gs: GameState, max_steps: u8) {
    let mut rand1 = SmallRng::seed_from_u64(100);
    let deck1 = sample_deck();
    let deck2 = sample_deck();
    let d1 = Decklist::new(Default::default(), deck1);
    let d2 = Decklist::new(Default::default(), deck2);
    let state = StandardNondetHandlerState::new(&d1, &d2, SmallRng::seed_from_u64(100).into());
    let mut nd = NondetProvider::new(state);
    'a: for i in 0..max_steps {
        while gs.to_move_player().is_none() {
            if let Phase::WinnerDecided { .. } = gs.phase {
                break 'a;
            }
            let input = nd.get_no_to_move_player_input(&gs);
            gs.advance(input).unwrap();
        }
        let actions = gs.available_actions();
        let input = actions[(rand1.next_u32() as usize) % actions.len()];
        gs.advance(input).unwrap();
        let h_complete = {
            let mut h = ZobristHasher::default();
            gs.zobrist_hash_full_recompute(&mut h);
            h.finish()
        };
        let h_incremental = gs.zobrist_hash();
        println!("--> {i} {input:?}");
        assert_eq!(h_complete, h_incremental);
    }
}
