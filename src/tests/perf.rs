use std::{hint::black_box, time::Instant};

use crate::zobrist_hash::ZobristHasher;

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

fn game_state() -> GameState {
    let mut gs = GameStateInitializer::new_skip_to_roll_phase(
        vector![CharId::Yoimiya, CharId::KamisatoAyaka, CharId::Xingqiu],
        vector![CharId::Fischl, CharId::Ningguang, CharId::Noelle],
    )
    .build();

    gs.advance_roll_phase_no_dice();
    gs.players.0.dice.add_single(Dice::Omni, 8);
    gs.players.1.dice.add_single(Dice::Omni, 8);
    gs
}

fn game_state_for_zobrist_hash() -> GameState {
    let mut gs = game_state();
    gs.players.0.add_to_hand_ignore(CardId::TheBestestTravelCompanion);
    gs.players.0.add_to_hand_ignore(CardId::SacrificialBow);
    gs.players.0.add_to_hand_ignore(CardId::LeaveItToMe);
    gs.players.0.add_to_hand_ignore(CardId::SacrificialBow);
    gs.advance_multiple([
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::NiwabiFireDance)),
        Input::FromPlayer(PlayerId::PlayerSecond, PlayerAction::CastSkill(SkillId::Nightrider)),
        Input::FromPlayer(PlayerId::PlayerFirst, PlayerAction::CastSkill(SkillId::FireworkFlareUp)),
    ]);
    gs
}

#[test]
fn bench_cast_skill() {
    let gs = game_state();
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
    let mut gs = game_state();
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
    let mut gs = game_state();
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
    let gs = game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.player(PlayerId::PlayerFirst)
                .zobrist_hash_full_recompute(&mut h, PlayerId::PlayerFirst);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_char_states() {
    let gs = game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.player(PlayerId::PlayerFirst)
                .zobrist_hash_for_char_states(&mut h, PlayerId::PlayerFirst);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_single_char_state() {
    let gs = game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.player(PlayerId::PlayerFirst)
                .active_character()
                .zobrist_hash(&mut h, PlayerId::PlayerFirst, 0);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_status_collection() {
    let gs = game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.status_collection(PlayerId::PlayerFirst)
                .zobrist_hash(&mut h, PlayerId::PlayerFirst);
            h.finish()
        });
    })
}

#[test]
fn bench_zobrist_hash_for_dice() {
    let gs = game_state_for_zobrist_hash();
    iter(|| {
        black_box({
            let mut h = ZobristHasher::new();
            gs.player(PlayerId::PlayerFirst)
                .zobrist_hash_for_dice(&mut h, PlayerId::PlayerFirst);
            h.finish()
        });
    })
}
