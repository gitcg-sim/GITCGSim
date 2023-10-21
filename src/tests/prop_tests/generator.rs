use enum_map::Enum;

use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

use crate::builder::GameStateBuilder;
use crate::cards::ids::{CardId, SummonId};
use crate::types::input::{Input, PlayerAction};
use crate::types::{game_state::*, nondet::*};
use crate::{cards::ids::CharId, data_structures::*, deck::Decklist, game_tree_search::*, vector};

pub fn arb_char_id() -> impl Strategy<Value = CharId> {
    (0..CharId::LENGTH).prop_map(CharId::from_usize)
}

pub fn arb_char_ids() -> impl Strategy<Value = Vector<CharId>> {
    (1..=3, arb_char_id(), arb_char_id(), arb_char_id())
        .prop_filter("Characters must be distinct.", |(_, a, b, c)| {
            a != b && a != c && b != c
        })
        .prop_map(|(n, a, b, c)| -> Vector<CharId> {
            let mut v = vector![];
            v.push(a);
            if n >= 2 {
                v.push(b);
            }
            if n >= 3 {
                v.push(c);
            }
            v
        })
}

prop_compose! {
    pub fn arb_init_game_state()(p1_chars in arb_char_ids(), p2_chars in arb_char_ids()) -> GameState {
        GameStateBuilder::default()
            .with_characters(p1_chars, p2_chars)
            .start_at_select_character()
            .build()
    }
}

prop_compose! {
    pub fn arb_deck()(count in 0..30, seed in any::<u64>()) -> smallvec::SmallVec<[CardId; 32]> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut cards: smallvec::SmallVec<[CardId; 32]> = Default::default();
        for _ in 0..count {
            loop {
                let card_id = CardId::from_usize(rng.gen_range(0..CardId::LENGTH));
                if card_id == CardId::BlankCard {
                    continue
                }
                cards.push(card_id);
                break
            }
        }
        cards
    }
}

prop_compose! {
    /// Deck does not validate.
    pub fn arb_decklist()(chars in arb_char_ids(), cards in arb_deck()) -> Decklist {
        Decklist::new(chars, cards)
    }
}

prop_compose! {
    pub fn arb_init_game_state_wrapper()(
        seed in any::<u64>(),
        decklist1 in arb_decklist(),
        decklist2 in arb_decklist()
    ) -> GameStateWrapper<StandardNondetHandlerState> {
        new_standard_game(&decklist1, &decklist2, SmallRng::seed_from_u64(seed))
    }
}

prop_compose! {
    pub fn arb_reachable_game_state_wrapper()(steps in 0..50, seed in any::<u64>(), init_gs in arb_init_game_state_wrapper()) -> GameStateWrapper<StandardNondetHandlerState> {
        let mut gs = init_gs;
        let mut rng = SmallRng::seed_from_u64(seed);
        for _ in 0..steps {
            let Some(..) = gs.to_move() else { break; };
            let acts = gs.actions();
            let act = acts[rng.gen_range(0..acts.len())];
            if let Err(e) = gs.advance(act) {
                dbg!(&gs);
                dbg!(&act);
                panic!("{e:?}");
            }
        }
        gs.game_state.rehash();
        gs
    }
}

prop_compose! {
    pub fn arb_init_game_state_wrapper_with_action()(gs in arb_init_game_state_wrapper(), seed in any::<u64>())
        -> (GameStateWrapper<StandardNondetHandlerState>, Input) {
        let actions = gs.actions();
        let mut rng = SmallRng::seed_from_u64(seed);
        (gs, actions[rng.gen_range(0..actions.len())])
    }
}

prop_compose! {
    pub fn arb_reachable_game_state_wrapper_with_action()(gs in arb_reachable_game_state_wrapper(), seed in any::<u64>())
        -> (GameStateWrapper<StandardNondetHandlerState>, Input) {
        let actions = gs.actions();
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut gs = gs;
        gs.game_state.ignore_costs = true;
        (gs, actions[rng.gen_range(0..actions.len())])
    }
}

prop_compose! {
    pub fn arb_reachable_game_state()(game_state_wrapper in arb_reachable_game_state_wrapper()) -> GameState {
        game_state_wrapper.game_state
    }
}

prop_compose! {
    pub fn arb_reachable_game_state_winner()(seed in any::<u64>(), game_state_wrapper in arb_reachable_game_state_wrapper()) -> GameStateWrapper<StandardNondetHandlerState> {
        let mut gs = game_state_wrapper;
        let mut rng = SmallRng::seed_from_u64(seed);
        let steps = 1000;
        for _ in 0..steps {
            let Some(..) = gs.to_move() else { return gs; };
            let acts = gs.actions();
            gs.advance(acts[rng.gen_range(0..acts.len())]).unwrap();
        }
        panic!("arb_reachable_game_state_winner: unable to reach a winner state.")
    }
}

pub fn arb_player_id() -> impl Strategy<Value = PlayerId> {
    prop_oneof![Just(PlayerId::PlayerFirst), Just(PlayerId::PlayerSecond)]
}

fn arb_char_idx() -> impl Strategy<Value = u8> {
    0u8..=3u8
}

fn arb_summon_id() -> impl Strategy<Value = SummonId> {
    (0..SummonId::LENGTH).prop_map(SummonId::from_usize)
}

fn arb_card_id() -> impl Strategy<Value = CardId> {
    (0..CardId::LENGTH).prop_map(CardId::from_usize)
}

fn arb_card_sel() -> impl Strategy<Value = Option<CardSelection>> {
    prop_oneof! [
        5 => Just(None),
        2 => arb_char_idx().prop_map(|i| Some(CardSelection::OwnCharacter(i))),
        1 => arb_summon_id().prop_map(|i| Some(CardSelection::OwnSummon(i))),
        1 => arb_summon_id().prop_map(|i| Some(CardSelection::OpponentSummon(i))),
    ]
}

fn arb_char_selection() -> impl Strategy<Value = CharSelection> {
    arb_char_idx()
}

pub fn arb_player_action() -> impl Strategy<Value = PlayerAction> {
    prop_oneof![
        Just(PlayerAction::EndRound),
        (arb_card_id(), arb_card_sel()).prop_map(|(card_id, card_sel)| PlayerAction::PlayCard(card_id, card_sel)),
        arb_card_id().prop_map(PlayerAction::ElementalTuning),
        arb_char_selection().prop_map(PlayerAction::SwitchCharacter),
        arb_char_selection().prop_map(PlayerAction::PostDeathSwitch),
    ]
}
