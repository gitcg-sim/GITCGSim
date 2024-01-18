use std::{hint::black_box, time::Instant};

use crate::zobrist_hash::ZobristHasher;

use self::prop_tests::generator::arb_reachable_game_state;

use super::*;

fn iter<F: Fn()>(f: F) {
    let t1 = Instant::now();
    let runs = 1_000_000;
    for _ in 0..runs {
        f()
    }

    let dt_ns = t1.elapsed().as_nanos();
    let dt_s = 1e-9 * (dt_ns as f64);
    println!("dt = {dt_s} s, dt/run = {} ns", (dt_ns as f64) / (runs as f64));
}

fn get_game_state() -> GameState {
    let mut gs = GameStateBuilder::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Xingqiu],
        vector![CharId::Fischl, CharId::Ningguang, CharId::Noelle],
    )
    .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.dice.add_in_place(&DiceCounter::omni(8));
    gs.players.1.dice.add_in_place(&DiceCounter::omni(8));
    gs
}

fn get_game_state_for_zobrist_hash() -> GameState {
    let mut gs = get_game_state();
    gs.players.0.hand.push(CardId::TheBestestTravelCompanion);
    gs.players.0.hand.push(CardId::SacrificialBow);
    gs.players.0.hand.push(CardId::LeaveItToMe);
    gs.players.0.hand.push(CardId::SacrificialBow);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    gs
}

fn get_random_game_states() -> smallvec::SmallVec<[GameState; 15]> {
    use proptest::prelude::*;
    use proptest::{
        strategy::ValueTree,
        test_runner::{TestRng, TestRunner},
    };
    let arb = arb_reachable_game_state();
    let config: ProptestConfig = Default::default();
    let mut runner = TestRunner::new(config);
    let rng = runner.rng();
    *rng = TestRng::from_seed(
        proptest::test_runner::RngAlgorithm::XorShift,
        &[123, 45, 67, 89, 0, 2, 5, 10, 123, 45, 67, 89, 0, 2, 5, 10],
    );
    rng.gen_bool(0.5);
    let tree = arb.new_tree(&mut runner).unwrap();
    (0..16).map(|_| tree.current()).collect()
}

#[test]
fn bench_cast_skill() {
    let gs = get_game_state();
    iter(|| {
        let mut gs = gs.clone();
        black_box(
            gs.advance(Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::CastSkill(SkillId::NiwabiFireDance),
            ))
            .unwrap(),
        );
    })
}

#[test]
fn bench_yoimiya_na_after_niwabi_fire_dance() {
    let mut gs = get_game_state();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
    ]);
    iter(|| {
        let mut gs = gs.clone();
        black_box(
            gs.advance(Input::FromPlayer(
                PlayerId::PlayerFirst,
                PlayerAction::CastSkill(SkillId::FireworkFlareUp),
            ))
            .unwrap(),
        );
    })
}

#[test]
fn bench_zobrist_hash() {
    let mut gs = get_game_state();
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.zobrist_hash_full_recompute(&mut h);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_player_state() {
    let gs = get_game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.get_player(PlayerId::PlayerFirst)
                .zobrist_hash_full_recompute(&mut h, PlayerId::PlayerFirst);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_char_states() {
    let gs = get_game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.get_player(PlayerId::PlayerFirst)
                .zobrist_hash_for_char_states(&mut h, PlayerId::PlayerFirst);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_single_char_state() {
    let gs = get_game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.get_player(PlayerId::PlayerFirst)
                .get_active_character()
                .zobrist_hash(&mut h, PlayerId::PlayerFirst, 0);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_status_collection() {
    let gs = get_game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.get_player(PlayerId::PlayerFirst)
                .status_collection
                .zobrist_hash(&mut h, PlayerId::PlayerFirst);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_dice() {
    let gs = get_game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.get_player(PlayerId::PlayerFirst)
                .zobrist_hash_for_dice(&mut h, PlayerId::PlayerFirst);
            h.finish()
        });
    })
}

mod available_actions {
    use super::*;
    fn bench_iter_available_actions<const N: usize, A: smallvec::Array<Item = Input>>() {
        let gss = get_random_game_states();
        iter(move || {
            for gs in &gss {
                black_box(gs.iter_available_actions().take(N).collect::<smallvec::SmallVec<A>>());
            }
        })
    }

    fn bench_available_actions<const N: usize, A: smallvec::Array<Item = Input>>() {
        let gss = get_random_game_states();
        iter(move || {
            for gs in &gss {
                black_box(
                    gs.available_actions()
                        .iter()
                        .take(N)
                        .copied()
                        .collect::<smallvec::SmallVec<A>>(),
                );
            }
        })
    }

    #[test]
    fn bench_first_1_available_actions() {
        bench_available_actions::<1, [Input; 1]>();
    }

    #[test]
    fn bench_first_1_iter_available_actions() {
        bench_iter_available_actions::<1, [Input; 1]>();
    }

    #[test]
    fn bench_first_4_available_actions() {
        bench_available_actions::<4, [Input; 4]>();
    }

    #[test]
    fn bench_first_4_iter_available_actions() {
        bench_iter_available_actions::<4, [Input; 4]>();
    }

    #[test]
    fn bench_first_8_available_actions() {
        bench_available_actions::<8, [Input; 8]>();
    }

    #[test]
    fn bench_first_8_iter_available_actions() {
        bench_iter_available_actions::<8, [Input; 8]>();
    }
}
