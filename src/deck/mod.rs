use crate::std_subset::{sync::Arc, String, Vec};

use enum_map::Enum;
use rand::Rng;

use smallvec::{smallvec, SmallVec};

use crate::cards::ids::*;

type DeckVec<T> = SmallVec<[T; 32]>;

mod parser;
pub use parser::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decklist {
    pub characters: SmallVec<[CharId; 4]>,
    pub cards: DeckVec<CardId>,
}

impl Decklist {
    pub fn new(characters: SmallVec<[CharId; 4]>, cards: DeckVec<CardId>) -> Self {
        Self { characters, cards }
    }
}

pub fn sample_deck() -> DeckVec<CardId> {
    smallvec![
        CardId::BrokenRimesEcho,
        CardId::ChangingShifts,
        CardId::ChangingShifts,
        CardId::DawnWinery,
        CardId::DawnWinery,
        CardId::ElementalResonanceWovenFlames,
        CardId::ElementalResonanceWovenFlames,
        CardId::ElementalResonanceWovenIce,
        CardId::ElementalResonanceWovenThunder,
        CardId::IHaventLostYet,
        CardId::IHaventLostYet,
        CardId::Katheryne,
        CardId::Katheryne,
        CardId::LeaveItToMe,
        CardId::LeaveItToMe,
        CardId::MondstadtHashBrown,
        CardId::MondstadtHashBrown,
        CardId::MushroomPizza,
        CardId::MushroomPizza,
        CardId::Paimon,
        CardId::Paimon,
        CardId::SacrificialSword,
        CardId::Starsigns,
        CardId::Starsigns,
        CardId::SweetMadame,
        CardId::SweetMadame,
        CardId::TheBestestTravelCompanion,
        CardId::TheBestestTravelCompanion,
        CardId::Strategize,
        CardId::Strategize,
    ]
}

fn gen_deck<R: Rng, T: PartialEq + Eq + Copy + Enum, A: smallvec::Array<Item = T>>(
    rand: &mut R,
    card_count: usize,
    max_copies: usize,
) -> SmallVec<A> {
    let mut v: SmallVec<A> = Default::default();
    let n = T::LENGTH;
    loop {
        let mut changed = false;
        for _ in 0..5 {
            let card_id: T = T::from_usize(rand.gen_range(0..n));
            let c = v.iter().copied().filter(|&c| c == card_id).count();
            if c < max_copies {
                v.push(card_id);
                changed = true;
                break;
            }
        }

        if v.len() >= card_count || !changed {
            break;
        }
    }
    v
}

fn random_chars<R: Rng>(rand: &mut R) -> SmallVec<[CharId; 4]> {
    gen_deck(rand, 3, 1)
}

fn random_deck<R: Rng>(rand: &mut R) -> SmallVec<[CardId; 32]> {
    gen_deck(rand, 30, 2)
}

pub fn random_decklist<R: Rng>(rand: &mut R) -> Decklist {
    Decklist::new(random_chars(rand), random_deck(rand))
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct DeckState {
    pub deck: Arc<DeckVec<CardId>>,
    pub mask: u64,
    pub count: u8,
}

impl DeckState {
    pub fn new(decklist: &Decklist) -> Self {
        let deck = Arc::new(decklist.cards.clone());
        let count = deck.len() as u8;
        let mask = if count == 0 { 0 } else { (1 << count) - 1 };
        Self { deck, mask, count }
    }

    #[inline]
    pub fn draw<R: Rng>(&mut self, rng: &mut R) -> Option<CardId> {
        if self.count == 0 {
            return None;
        }

        let i = rng.gen_range(0..self.count);
        let n = self.deck.len() as u8;
        let mask = self.mask;
        let Some(k) = (0..n).filter(|j| mask & (1 << j) != 0).nth(i as usize) else {
            return None;
        };
        self.mask ^= 1 << k;
        self.count -= 1;
        Some(self.deck[k as usize])
    }
}
