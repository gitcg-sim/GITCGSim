use std::{
    cmp::min,
    hash::{Hash, Hasher},
};
// use std::{collections::hash_map::FxHasher, hash::{Hash, Hasher}};
// use rand::prelude::*;

use enumset::{enum_set, EnumSet, EnumSetType};
use rand::{Rng, RngCore};
use rustc_hash::FxHasher;

use smallvec::SmallVec;

use crate::{
    cards::ids::*,
    data_structures::{capped_list::CappedLengthList8, Vector},
    deck::*,
    dispatcher_ops::types::NondetRequest,
    game_tree_search::ZobristHashable,
    list8,
    rng::RngState,
    zobrist_hash::HashValue,
};

use super::{
    command::SummonRandomSpec,
    dice_counter::{distribution::DiceDistribution, DiceCounter},
    game_state::{GameState, PlayerId},
    input::{Input, NondetResult},
};

/// Trait for handling non-deterministic aspects of the game such as Elemental Dice rolls and drawing cards.
pub trait NondetState: ZobristHashable + Clone + Send + Sync {
    #[allow(unused_variables)]
    fn hide_private_information(&mut self, private_player_id: PlayerId, game_state: &mut GameState) {}

    fn sample_nondet(&mut self, game_state: &GameState, req: NondetRequest) -> NondetResult;
    //fn sample_nondet_multi(&mut self, game_state: &GameState, request: NondetRequest, samples: u8) -> Vec<(f32, NondetResult)>;
}

/// Provides no cards and no dice.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EmptyNondetState();

impl NondetState for EmptyNondetState {
    fn sample_nondet(&mut self, _game_state: &GameState, req: NondetRequest) -> NondetResult {
        match req {
            NondetRequest::DrawCards(_, _) => NondetResult::ProvideCards(list8![], list8![]),
            NondetRequest::RollDice(_, _) => NondetResult::ProvideDice(DiceCounter::EMPTY, DiceCounter::EMPTY),
            NondetRequest::DrawCardsOfType(_, _, _) => NondetResult::ProvideCards(list8![], list8![]),
            NondetRequest::SummonRandom(_) => NondetResult::ProvideSummonIds(list8![]),
        }
    }
}

impl ZobristHashable for EmptyNondetState {
    fn zobrist_hash(&self) -> HashValue {
        28432498
    }
}

#[derive(Debug, PartialOrd, Ord, EnumSetType)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[enumset(repr = "u8")]
pub enum StandardNondetHandlerFlags {
    PlayerFirstPrivate,
    PlayerFirstFuture,
    PlayerSecondPrivate,
    PlayerSecondFuture,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StandardNondetHandlerState {
    pub decks: (DeckState, DeckState),
    pub rng: RngState,
    pub flags: EnumSet<StandardNondetHandlerFlags>,
}

impl StandardNondetHandlerState {
    pub fn new(s1: &Decklist, s2: &Decklist, rng: RngState) -> Self {
        Self {
            decks: (DeckState::new(s1), DeckState::new(s2)),
            rng,
            flags: Default::default(),
        }
    }

    #[inline]
    fn should_hide_player_cards(&self, player_id: PlayerId) -> bool {
        player_id.select((
            self.flags.contains(StandardNondetHandlerFlags::PlayerFirstFuture)
                | self.flags.contains(StandardNondetHandlerFlags::PlayerFirstPrivate),
            self.flags.contains(StandardNondetHandlerFlags::PlayerSecondFuture)
                | self.flags.contains(StandardNondetHandlerFlags::PlayerSecondPrivate),
        ))
    }

    #[inline]
    fn should_hide_player_dice(&self, player_id: PlayerId) -> bool {
        self.should_hide_player_cards(player_id)
    }

    #[inline]
    fn roll_dice(&mut self, player_id: PlayerId, dist: DiceDistribution) -> DiceCounter {
        if self.should_hide_player_dice(player_id) {
            DiceCounter::simplified_dice(dist)
        } else {
            DiceCounter::rand_with_reroll(&mut self.rng, dist)
        }
    }

    fn draw_cards(&mut self, player_id: PlayerId, count: u8) -> CappedLengthList8<CardId> {
        if count >= 8 {
            unimplemented!();
        }
        let hide = self.should_hide_player_cards(player_id);
        let d = player_id.select_mut(&mut self.decks);
        let mut v = SmallVec::<[CardId; 8]>::with_capacity(min(8, count as usize));
        let range = 0..min(8, count);
        for _ in range {
            if let Some(c) = d.draw(&mut self.rng) {
                v.push(if hide { CardId::BlankCard } else { c })
            } else {
                break;
            }
        }
        CappedLengthList8::from(v)
    }
}

impl ZobristHashable for StandardNondetHandlerState {
    fn zobrist_hash(&self) -> HashValue {
        let mut h = FxHasher::default();
        self.decks.0.mask.hash(&mut h);
        self.decks.1.mask.hash(&mut h);
        let mut rng = self.rng.clone();
        h.finish() ^ rng.next_u64()
    }
}

impl NondetState for StandardNondetHandlerState {
    fn hide_private_information(&mut self, private_player_id: PlayerId, game_state: &mut GameState) {
        self.flags =
            enum_set![StandardNondetHandlerFlags::PlayerFirstFuture | StandardNondetHandlerFlags::PlayerSecondFuture];
        self.flags.insert(private_player_id.select((
            StandardNondetHandlerFlags::PlayerFirstPrivate,
            StandardNondetHandlerFlags::PlayerSecondPrivate,
        )));

        let player = game_state.get_player_mut(private_player_id);
        for c in player.hand.iter_mut() {
            *c = CardId::BlankCard;
        }
        player.flags.insert(super::game_state::PlayerFlag::Tactical);
        player.dice = DiceCounter::rand_with_reroll(&mut self.rng, player.get_dice_distribution());
        game_state.rehash();
    }

    fn sample_nondet(&mut self, _game_state: &GameState, req: NondetRequest) -> NondetResult {
        match req {
            NondetRequest::DrawCards(a, b) => NondetResult::ProvideCards(
                self.draw_cards(PlayerId::PlayerFirst, a),
                self.draw_cards(PlayerId::PlayerSecond, b),
            ),
            NondetRequest::DrawCardsOfType(player_id, count, card_type) => {
                if card_type.is_some() {
                    todo!()
                }
                match player_id {
                    PlayerId::PlayerFirst => {
                        NondetResult::ProvideCards(self.draw_cards(PlayerId::PlayerFirst, count), list8![])
                    }
                    PlayerId::PlayerSecond => {
                        NondetResult::ProvideCards(list8![], self.draw_cards(PlayerId::PlayerSecond, count))
                    }
                }
            }
            NondetRequest::RollDice(d1, d2) => NondetResult::ProvideDice(
                self.roll_dice(PlayerId::PlayerFirst, d1),
                self.roll_dice(PlayerId::PlayerSecond, d2),
            ),
            NondetRequest::SummonRandom(spec) => NondetResult::ProvideSummonIds(spec.sample(&mut self.rng)),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NondetProvider<S: NondetState = StandardNondetHandlerState> {
    pub state: S,
}

impl<S: NondetState> NondetProvider<S> {
    #[inline]
    pub fn new(state: S) -> Self {
        Self { state }
    }

    /// Precondition: game_state.to_move_player().is_none()
    pub fn get_no_to_move_player_input(&mut self, game_state: &GameState) -> Input {
        if let Some(req) = game_state.get_nondet_request() {
            Input::NondetResult(S::sample_nondet(&mut self.state, game_state, req))
        } else {
            Input::NoAction
        }
    }

    pub fn hide_private_information(&mut self, game_state: &mut GameState, private_player_id: PlayerId) {
        S::hide_private_information(&mut self.state, private_player_id, game_state)
    }
}

impl<S: NondetState> ZobristHashable for NondetProvider<S> {
    #[inline]
    fn zobrist_hash(&self) -> HashValue {
        self.state.zobrist_hash()
    }
}

impl SummonRandomSpec {
    fn sample<R: Rng>(&self, r: &mut R) -> CappedLengthList8<SummonId> {
        let Self {
            summon_ids,
            existing_summon_ids,
            count,
        } = *self;
        let count = count as usize;
        let mut to_summon: EnumSet<SummonId> = summon_ids.to_enum_set();
        for _ in 0..summon_ids.len() {
            if to_summon.len() <= count {
                break;
            }
            let to_remove = {
                let intersect = existing_summon_ids & to_summon;
                if intersect.is_empty() {
                    to_summon
                } else {
                    intersect
                }
            };
            if to_remove.is_empty() {
                break;
            }
            let Some(randomly_selected) = to_remove.iter().nth(r.gen_range(0..to_remove.len())) else {
                break;
            };
            to_summon.remove(randomly_selected);
        }
        let v: Vector<_> = to_summon.iter().collect();
        v.into()
    }
}

#[cfg(test)]
mod summon_random_sample_test {
    use super::*;
    use enumset::enum_set;
    use rand::{rngs::SmallRng, SeedableRng};

    use crate::types::command::SummonRandomSpec;

    const SUMMON_IDS: CappedLengthList8<SummonId> = list8![
        SummonId::OceanidMimicSquirrel,
        SummonId::OceanidMimicRaptor,
        SummonId::OceanidMimicFrog
    ];

    #[test]
    fn test_empty_result() {
        let spec = SummonRandomSpec::new(
            SUMMON_IDS,
            enum_set![SummonId::OceanidMimicRaptor | SummonId::OceanidMimicFrog],
            0,
        );
        assert_eq!(enum_set![], spec.sample(&mut SmallRng::seed_from_u64(0)).to_enum_set());

        let spec = SummonRandomSpec::new(
            SUMMON_IDS,
            enum_set![SummonId::OceanidMimicRaptor | SummonId::OceanidMimicFrog | SummonId::OceanidMimicSquirrel],
            0,
        );
        assert_eq!(enum_set![], spec.sample(&mut SmallRng::seed_from_u64(0)).to_enum_set());

        let spec = SummonRandomSpec::new(SUMMON_IDS, enum_set![], 0);
        assert_eq!(enum_set![], spec.sample(&mut SmallRng::seed_from_u64(0)).to_enum_set());

        let spec = SummonRandomSpec::new(
            SUMMON_IDS,
            enum_set![
                SummonId::OceanidMimicRaptor
                    | SummonId::OceanidMimicFrog
                    | SummonId::OceanidMimicSquirrel
                    | SummonId::Oz
            ],
            0,
        );
        assert_eq!(enum_set![], spec.sample(&mut SmallRng::seed_from_u64(0)).to_enum_set());
    }

    #[test]
    fn test_unique_solution() {
        let spec = SummonRandomSpec::new(
            SUMMON_IDS,
            enum_set![SummonId::OceanidMimicRaptor | SummonId::OceanidMimicFrog],
            1,
        );
        assert_eq!(
            enum_set![SummonId::OceanidMimicSquirrel],
            spec.sample(&mut SmallRng::seed_from_u64(0)).to_enum_set()
        );

        let spec = SummonRandomSpec::new(
            list8![SummonId::OceanidMimicRaptor],
            enum_set![SummonId::OceanidMimicRaptor],
            1,
        );
        assert_eq!(
            enum_set![SummonId::OceanidMimicRaptor],
            spec.sample(&mut SmallRng::seed_from_u64(0)).to_enum_set()
        );

        let spec = SummonRandomSpec::new(SUMMON_IDS, enum_set![SummonId::OceanidMimicRaptor], 2);
        assert_eq!(
            enum_set![SummonId::OceanidMimicSquirrel | SummonId::OceanidMimicFrog],
            spec.sample(&mut SmallRng::seed_from_u64(0)).to_enum_set()
        );

        let spec = SummonRandomSpec::new(SUMMON_IDS, enum_set![SummonId::OceanidMimicFrog], 2);
        assert_eq!(
            enum_set![SummonId::OceanidMimicSquirrel | SummonId::OceanidMimicRaptor],
            spec.sample(&mut SmallRng::seed_from_u64(0)).to_enum_set()
        );
    }

    #[test]
    fn test_prioritize_new_full_coverage() {
        let mut r = SmallRng::seed_from_u64(0);
        for _ in 0..10 {
            let spec = SummonRandomSpec::new(SUMMON_IDS, enum_set![SummonId::OceanidMimicFrog], 1);
            let res = spec.sample(&mut r).to_enum_set();
            assert_eq!(1, res.len());
            assert_eq!(enum_set![], res & enum_set![SummonId::OceanidMimicFrog]);

            let spec = SummonRandomSpec::new(
                list8![
                    SummonId::Oz,
                    SummonId::OceanidMimicFrog,
                    SummonId::OceanidMimicRaptor,
                    SummonId::OceanidMimicSquirrel
                ],
                enum_set![SummonId::OceanidMimicFrog | SummonId::Oz],
                2,
            );
            let res = spec.sample(&mut r).to_enum_set();
            assert_eq!(2, res.len());
            assert_eq!(enum_set![], res & enum_set![SummonId::OceanidMimicFrog | SummonId::Oz]);
        }
    }

    #[test]
    fn test_prioritize_new_including_existing() {
        let mut r = SmallRng::seed_from_u64(0);
        for _ in 0..10 {
            let spec = SummonRandomSpec::new(
                SUMMON_IDS,
                enum_set![SummonId::OceanidMimicFrog | SummonId::OceanidMimicRaptor],
                2,
            );
            let res = spec.sample(&mut r).to_enum_set();
            assert_eq!(2, res.len());
            assert_ne!(
                enum_set![],
                res & enum_set![SummonId::OceanidMimicFrog | SummonId::OceanidMimicRaptor]
            );
        }
    }
}
