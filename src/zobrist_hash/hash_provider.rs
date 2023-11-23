use std::hash::Hasher;

use rand::prelude::*;
use rand::rngs::SmallRng;
use rustc_hash::FxHasher;

use crate::dispatcher_ops::types::NondetRequest;
use crate::types::by_player::ByPlayer;
use crate::types::ElementSet;

use super::*;

/// Random seed used to compute Zobrist hashes.
const SEED: u64 = 1200;

pub const HP_COUNT: usize = 32;
pub const ENERGY_COUNT: usize = 4;
pub const CHAR_COUNT: usize = 8;

/// Max number of cards on hand generated for the hasher.
pub const CARD_COUNT: usize = 4;

/// Max possible dice count generated for the hasher.
pub const DICE_COUNT: usize = 8;

lazy_static! {
    /// The default instance for the `HashProvider`.
    pub static ref HASH_PROVIDER: HashProvider = HashProvider::new();
}

/// Number of possible status usage/durations
pub const STATUS_A_COUNT: usize = 8;
/// Number of possible status once per round * counter combinations
pub const STATUS_B_COUNT: usize = 32;

type BoxStatusHashes = Box<[[HashValue; STATUS_B_COUNT]; STATUS_A_COUNT]>;

/// Contains pre-computed Zobrist hashes of individual elements of the `GameState`.
/// The random seed used to compute these hashes is `SEED`.
///
/// If there is an index overflow, a hash based on `FxHash` will be computed instad.
pub struct HashProvider {
    pub active_char_idx_hashes: ByPlayer<[HashValue; CHAR_COUNT]>,
    pub hp_hashes: ByPlayer<[[HashValue; HP_COUNT]; CHAR_COUNT]>,
    pub energy_hashes: ByPlayer<[[HashValue; ENERGY_COUNT]; CHAR_COUNT]>,
    pub applied_elements_hashes: ByPlayer<[[HashValue; 8]; CHAR_COUNT]>,
    pub char_flags_hashes: ByPlayer<[[[HashValue; 16]; 16]; CHAR_COUNT]>,
    pub total_dmg_taken_hashes: ByPlayer<[[[HashValue; 16]; 16]; CHAR_COUNT]>,
    // Since constructing a HashProvider requires the array to be allocated on the stack,
    // Box is used in some parts of the hash collections
    pub team_status_hashes: Box<ByPlayer<EnumMap<StatusId, BoxStatusHashes>>>,
    pub char_status_hashes: Box<ByPlayer<EnumMap<StatusId, [BoxStatusHashes; CHAR_COUNT]>>>,
    pub summon_hashes: Box<ByPlayer<EnumMap<SummonId, BoxStatusHashes>>>,
    pub support_hashes: Box<ByPlayer<EnumMap<SupportSlot, EnumMap<SupportId, BoxStatusHashes>>>>,
    pub card_hashes: ByPlayer<EnumMap<CardId, [HashValue; CARD_COUNT]>>,
    pub dice_hashes: ByPlayer<[[HashValue; DICE_COUNT]; 8]>,
    pub phase_hashes: [HashValue; 18],
    pub other_hashes: ByPlayer<[HashValue; 32]>,
}

macro_rules! by_player {
    ($body: expr) => {{
        ByPlayer($body, $body)
    }};
}

macro_rules! rand_array {
    ( [ $($rest: expr);+ ] ; $count: expr ) => {
        {
            let mut arr: [_; $count] = Default::default();
            for i in 0 .. $count {
                arr[i] = rand_array![$($rest);+];
            }
            arr
        }
    };
    ($expr: expr ; $count: expr $(;)?) => {
        {
            let mut arr: [_; $count] = Default::default();
            for i in 0 .. $count {
                arr[i] = $expr;
            }
            arr
        }
    };
}

macro_rules! by_enum {
    ($body: expr) => {{
        let mut m = EnumMap::default();
        for (_, v) in m.iter_mut() {
            *v = $body;
        }
        m
    }};
}

impl HashProvider {
    pub fn new() -> Self {
        let mut rng = SmallRng::seed_from_u64(SEED);
        #[cfg(HASH128)]
        macro_rules! random {
            () => {
                ((rng.next_u64() as u128) << 64) | (rng.next_u64() as u128)
            };
        }

        #[cfg(not(HASH128))]
        macro_rules! random {
            () => {
                rng.next_u64()
            };
        }

        macro_rules! bs {
            () => {
                Box::new(rand_array![[random!(); STATUS_B_COUNT]; STATUS_A_COUNT])
            };
        }

        let active_char_idx_hashes = by_player!(rand_array![random!(); CHAR_COUNT]);
        let hp_hashes = by_player!(rand_array![[random!(); HP_COUNT]; CHAR_COUNT]);
        let energy_hashes = by_player!(rand_array![[random!(); ENERGY_COUNT]; CHAR_COUNT]);
        let applied_elements_hashes = by_player!(rand_array![[random!(); 8]; CHAR_COUNT]);
        let char_flags_hashes = by_player!(rand_array![[[random!(); 16]; 16]; CHAR_COUNT]);
        let total_dmg_taken_hashes = by_player!(rand_array![[[random!(); 16]; 16]; CHAR_COUNT]);
        let team_status_hashes = Box::new(by_player!(by_enum!(bs!())));
        let char_status_hashes = Box::new(by_player!(by_enum!(rand_array![bs!(); CHAR_COUNT])));
        let summon_hashes = Box::new(by_player!(by_enum!(bs!())));
        let support_hashes = Box::new(by_player!(by_enum!(by_enum!(bs!()))));
        let card_hashes = by_player!(by_enum!(rand_array![random!(); CARD_COUNT]));
        let dice_hashes = by_player!(rand_array![[random!(); DICE_COUNT]; 8]);
        let phase_hashes = rand_array![random!(); 18];
        let other_hashes = by_player!(rand_array![random!(); 32]);
        Self {
            active_char_idx_hashes,
            hp_hashes,
            energy_hashes,
            applied_elements_hashes,
            char_flags_hashes,
            total_dmg_taken_hashes,
            team_status_hashes,
            char_status_hashes,
            summon_hashes,
            support_hashes,
            card_hashes,
            dice_hashes,
            phase_hashes,
            other_hashes,
        }
    }

    #[inline]
    pub fn phase(&self, phase: Phase) -> HashValue {
        self.phase_hashes[phase.to_index()]
    }

    #[inline]
    pub fn active_char_idx(&self, player_id: PlayerId, char_idx: u8) -> HashValue {
        self.active_char_idx_hashes[player_id][char_idx as usize]
    }

    #[inline]
    pub fn hp(&self, player_id: PlayerId, char_idx: u8, hp: u8) -> HashValue {
        self.hp_hashes[player_id][char_idx as usize][hp as usize]
    }

    #[inline]
    pub fn energy(&self, player_id: PlayerId, char_idx: u8, energy: u8) -> HashValue {
        self.energy_hashes[player_id][char_idx as usize][energy as usize]
    }

    #[inline]
    pub fn applied_elements(&self, player_id: PlayerId, char_idx: u8, elems: ElementSet) -> HashValue {
        if elems.is_empty() {
            return 0;
        }

        let idx = if elems.len() == 2 {
            // Cryo + Dendro
            7
        } else if elems.len() == 1 {
            let e = elems.iter().next().unwrap();
            e.to_index()
        } else {
            panic!("applied_elements: Invalid");
        };

        self.applied_elements_hashes[player_id][char_idx as usize][idx]
    }

    #[inline]
    pub fn char_flags(&self, player_id: PlayerId, char_idx: u8, flags: EnumSet<CharFlag>) -> HashValue {
        if flags.is_empty() {
            return 0;
        }

        let v: u8 = flags.as_repr();
        let (a, b) = (v / 16, v % 16);
        self.char_flags_hashes[player_id][char_idx as usize][a as usize][b as usize]
    }

    #[inline]
    pub fn total_dmg_taken(&self, player_id: PlayerId, char_idx: u8, value: u8) -> HashValue {
        let (a, b) = (value / 16, value % 16);
        self.total_dmg_taken_hashes[player_id][char_idx as usize][a as usize][b as usize]
    }

    #[inline]
    pub fn team_status(&self, player_id: PlayerId, status_id: StatusId, a: u8, b: u8) -> HashValue {
        self.team_status_hashes[player_id][status_id][a as usize][b as usize]
    }

    #[inline]
    pub fn summon_status(&self, player_id: PlayerId, summon_id: SummonId, a: u8, b: u8) -> HashValue {
        // TODO check out of bounds
        self.summon_hashes[player_id][summon_id][a as usize][b as usize]
    }

    #[inline]
    pub fn support_status(
        &self,
        player_id: PlayerId,
        slot: SupportSlot,
        support_id: SupportId,
        a: u8,
        b: u8,
    ) -> HashValue {
        // TODO check out of bounds
        self.support_hashes[player_id][slot][support_id][a as usize][b as usize]
    }

    #[inline]
    pub fn character_status(&self, player_id: PlayerId, char_idx: u8, status_id: StatusId, a: u8, b: u8) -> HashValue {
        // TODO check out of bounds
        self.char_status_hashes[player_id][status_id][char_idx as usize][a as usize][b as usize]
    }

    #[inline]
    pub fn hand(&self, player_id: PlayerId, card_id: CardId, count: u8) -> HashValue {
        if count == 0 {
            return 0;
        }

        let count = (count - 1) as usize;
        if count >= CARD_COUNT {
            let mut h = FxHasher::default();
            player_id.hash(&mut h);
            card_id.hash(&mut h);
            count.hash(&mut h);
            return h.finish() as HashValue;
        }

        self.card_hashes[player_id][card_id][count]
    }

    #[inline]
    pub fn dice(&self, player_id: PlayerId, dice: Dice, count: u8) -> HashValue {
        if count == 0 {
            return 0;
        }

        if (count as usize) < DICE_COUNT {
            return self.dice_hashes[player_id][dice.to_index()][count as usize];
        }

        // Fallback
        let mut h = FxHasher::default();
        player_id.hash(&mut h);
        dice.to_index().hash(&mut h);
        count.hash(&mut h);
        h.finish() as HashValue
    }

    #[inline]
    pub fn player_flags(&self, player_id: PlayerId, flags: EnumSet<PlayerFlag>) -> HashValue {
        let flags = flags.as_repr();
        if flags < 8 {
            return self.other_hashes[player_id][flags as usize];
        }

        // Fallback
        let mut h = FxHasher::default();
        flags.hash(&mut h);
        player_id.hash(&mut h);
        h.finish() as HashValue
    }

    #[inline]
    pub fn post_death_switch(&self, player_id: PlayerId) -> HashValue {
        self.other_hashes[player_id][2]
    }

    #[inline]
    pub fn nondet_request(&self, req: NondetRequest) -> HashValue {
        let mut h = FxHasher::default();
        req.hash(&mut h);
        h.finish() as HashValue
    }

    /// Given a hash value, modify it based on an "index" value.
    /// This function exists to avoid pre-computing Zobrist hash elements based on index.
    #[inline]
    pub fn with_index(hv: HashValue, index: usize) -> HashValue {
        if index <= 31 {
            hv.rotate_right(index as u32)
        } else {
            let mut h = FxHasher::default();
            hv.hash(&mut h);
            index.hash(&mut h);
            h.finish() as HashValue
        }
    }
}

impl Default for HashProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for HashProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashProvider").finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cards::ids::CardId, status_impls::prelude::Dice, types::game_state::PlayerId, zobrist_hash::HASH_PROVIDER,
    };

    #[test]
    fn test_macro_rules_for_rand_array_are_ordered_properly() {
        let arr = &HASH_PROVIDER.card_hashes[PlayerId::PlayerFirst][CardId::Paimon];
        assert_ne!(arr[0], arr[1]);
        let arr = &HASH_PROVIDER.dice_hashes[PlayerId::PlayerSecond][Dice::CRYO.to_index()];
        assert_ne!(arr[0], arr[1]);
    }
}
