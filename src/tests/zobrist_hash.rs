use rand::{rngs::SmallRng, RngCore, SeedableRng};

use crate::{
    deck::{sample_deck, Decklist},
    types::nondet::{NondetProvider, StandardNondetHandlerState},
    zobrist_hash::ZobristHasher,
};

use super::*;

#[test]
fn zobrist_hash() {
    let mut gs =
        GameStateInitializer::new_skip_to_roll_phase(vector![CharId::Fischl], vector![CharId::Kaeya, CharId::Yoimiya])
            .ignore_costs(true)
            .build();
    gs.advance_roll_phase_no_dice();
    gs.advance_multiple([
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
    gs.advance_multiple([Input::FromPlayer(
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
fn zobrist_hash_random_steps_1() {
    let gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Keqing, CharId::Yoimiya, CharId::Cyno],
        vector![CharId::Klee, CharId::Xingqiu, CharId::Mona],
    )
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

#[test]
fn zobrist_hash_random_steps_2() {
    let gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::KamisatoAyaka, CharId::Yoimiya, CharId::Collei],
        vector![CharId::Diona, CharId::Ningguang, CharId::Noelle],
    )
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

#[test]
fn zobrist_hash_random_steps_3() {
    let gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Barbara, CharId::Fischl, CharId::Collei],
        vector![CharId::Mona, CharId::Ganyu, CharId::Kaeya],
    )
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

#[test]
fn zobrist_hash_random_steps_4() {
    let gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::KujouSara, CharId::Keqing, CharId::SangonomiyaKokomi],
        vector![CharId::Mona, CharId::Cyno, CharId::Eula],
    )
    .build();
    _test_zobrist_hash_random_steps(gs, 200);
}

#[test]
fn zobrist_hash_random_steps_5() {
    let gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Eula, CharId::Fischl, CharId::Diona],
        vector![CharId::KamisatoAyaka, CharId::Ganyu, CharId::Xingqiu],
    )
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
            let input = nd.no_to_move_player_input(&gs);
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
