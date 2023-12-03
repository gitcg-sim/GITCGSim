use std::ops::Range;

use enum_map::Enum;

use proptest::prelude::*;
use rand::seq::SliceRandom;
use rand::{rngs::SmallRng, SeedableRng};

use crate::builder::GameStateBuilder;
use crate::cards::ids::{CardId, SummonId};
use crate::types::input::{Input, PlayerAction};
use crate::types::{game_state::*, nondet::*};
use crate::{cards::ids::CharId, data_structures::*, deck::Decklist, game_tree_search::*};

pub fn arb_enum<E: std::fmt::Debug + Enum>() -> impl Strategy<Value = E> {
    (0..E::LENGTH).prop_map(E::from_usize)
}

pub fn arb_distinct_vec<
    E: Copy + std::hash::Hash + std::fmt::Debug + std::cmp::Eq,
    T: Strategy<Value = E>,
    S: Into<proptest::sample::SizeRange>,
>(
    element: T,
    size: S,
) -> impl Strategy<Value = Vector<E>> {
    proptest::collection::hash_set(element, size.into())
        .prop_map(|s| s.iter().copied().collect())
        .prop_perturb(|mut v: Vector<E>, mut rng| {
            v.shuffle(&mut rng);
            v
        })
}

pub fn arb_char_id() -> impl Strategy<Value = CharId> {
    arb_enum()
}

pub fn arb_char_ids() -> impl Strategy<Value = Vector<CharId>> {
    arb_distinct_vec(arb_char_id(), 1..=3)
}

pub fn arb_char_ids_containing(char_id: CharId) -> impl Strategy<Value = Vector<CharId>> {
    arb_distinct_vec(arb_char_id(), 0..=2).prop_perturb(move |mut v: Vector<CharId>, mut rng| {
        v.push(char_id);
        v.shuffle(&mut rng);
        v
    })
}

prop_compose! {
    pub fn arb_init_game_state()(p1_chars in arb_char_ids(), p2_chars in arb_char_ids()) -> GameState {
        GameStateBuilder::default()
            .characters(p1_chars, p2_chars)
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

/// Deck does not validate.
pub fn arb_decklist_with_chars(chars: impl Strategy<Value = Vector<CharId>>) -> impl Strategy<Value = Decklist> {
    (chars, arb_deck()).prop_map(|(chars, cards)| Decklist::new(chars, cards))
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

pub struct ArbGameState<D1: Strategy<Value = Decklist>, D2: Strategy<Value = Decklist>> {
    pub arb_deck1: D1,
    pub arb_deck2: D2,
}

impl<D1: Strategy<Value = Decklist>, D2: Strategy<Value = Decklist>> ArbGameState<D1, D2> {
    pub fn new(arb_deck1: D1, arb_deck2: D2) -> Self {
        Self { arb_deck1, arb_deck2 }
    }

    // TODO remove this warning if needed
    #[allow(dead_code)]
    pub fn arb_game_state(self) -> impl Strategy<Value = GameState> {
        (self.arb_deck1, self.arb_deck2).prop_map(|(d1, d2)| {
            GameStateBuilder::default()
                .characters(d1.characters, d2.characters)
                .start_at_select_character()
                .build()
        })
    }

    pub fn arb_game_state_wrapper(self) -> impl Strategy<Value = GameStateWrapper<StandardNondetHandlerState>> {
        (self.arb_deck1, self.arb_deck2, any::<u64>()).prop_map(|(d1, d2, rng)| {
            let state = StandardNondetHandlerState::new(&d1, &d2, SmallRng::seed_from_u64(rng).into());
            let gs = GameStateBuilder::default()
                .characters(d1.characters, d2.characters)
                .skip_to_roll_phase()
                .build();
            GameStateWrapper::new(gs, NondetProvider::new(state))
        })
    }

    pub fn arb_reachable(self) -> ArbReachableGameStateWrapper<impl Strategy<Value = GameStateWrapper>> {
        ArbReachableGameStateWrapper::new(self.arb_game_state_wrapper())
    }
}

pub struct ArbReachableGameStateWrapper<T: Strategy<Value = GameStateWrapper<StandardNondetHandlerState>>> {
    pub steps: Range<usize>,
    pub arb_init_game_state_wrapper: T,
    pub arb_seed: <u64 as Arbitrary>::Strategy,
}

impl<T: Strategy<Value = GameStateWrapper<StandardNondetHandlerState>>> ArbReachableGameStateWrapper<T> {
    const MAX_STEPS: usize = 50usize;
    pub fn new(arb_init_game_state_wrapper: T) -> Self {
        Self {
            steps: 0..Self::MAX_STEPS,
            arb_init_game_state_wrapper,
            arb_seed: u64::arbitrary(),
        }
    }

    pub fn arb(self) -> impl Strategy<Value = GameStateWrapper<StandardNondetHandlerState>> {
        (self.steps, self.arb_seed, self.arb_init_game_state_wrapper).prop_map(|(steps, seed, mut gs)| {
            let mut rng = SmallRng::seed_from_u64(seed);
            for _ in 0usize..steps {
                let Some(..) = gs.to_move() else {
                    break;
                };
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
        })
    }
}

fn default_arb_game_state_config(
) -> ArbReachableGameStateWrapper<impl Strategy<Value = GameStateWrapper<StandardNondetHandlerState>>> {
    ArbReachableGameStateWrapper::new(arb_init_game_state_wrapper())
}

pub fn arb_reachable_game_state_wrapper() -> impl Strategy<Value = GameStateWrapper<StandardNondetHandlerState>> {
    default_arb_game_state_config().arb()
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
