/// Zobrist hashing is the hashing method used for Genius Invokation TCG game states.
/// The Zobrist hash of a game state is the XOR over all aspects of the game state that
/// is relevant to its distinctiveness, such as:
/// - The first character of the first player has HP of 6
/// - There are 2 copies of "Strategize" on the second player's hand.
///
/// The Zobrist hash can be updated for each incremental change of the game state.
///
/// Wikipedia: https://en.wikipedia.org/wiki/Zobrist_hashing
/// Chess Programming Wiki: https://www.chessprogramming.org/Zobrist_Hashing
///
/// # Zobrist hashing and datatypes
/// ## Equality
/// For a type `T` to be Zobrist hashable, it must be isomorphic to a finite set of features `features: T -> Set<F>`.
///
/// Then the equality for `T` can be defined into:
///
/// ```text
/// eq(a: T, b:T) := features(a) == features(b)
/// ```
///
/// Then the Zobrist hash for `T` is:
///
/// ```text
/// hash(a: T) := xor( { hash_element(e) | e in to_set(a) })
/// ```
///
/// ## Multisets
/// For a collection of values where the order does not matter but
/// multiplicity (number of copies of an element) matters:
/// ```text
/// features(xs: List<F>): Set<(F, number)> := { (e, xs.multiplicity(e)) | e in xs }
/// ```
///
/// In the Genius Invokation TCG, a player's dice and hand are hashed as multisets.
///
/// ## Ordered lists
/// For a list where the order of elements matter:
/// ```text
/// features(xs: List<F>): Set<(F, number)> := { (e, index) | (index, e) in enumerate(xs) }
/// ```
///
/// ## Structs and enums
/// The features are tupled with the path to the struct/enum item.
/// ```text
/// // Sum type (enums)
/// features(xs: A + B + ...): Set<(F, number)> = { (a, 0) | a in features(A) } union { (b, 1) b in features(B) } union ...
/// // Product type (structs and tuples)
/// features(xs: A * B * ...): Set<(F, number)> = { (a, 0) | a in features(A) } union { (b, 1) b in features(B) } union ...
/// ```
///
use std::fmt::Debug;
use std::hash::Hash;

use enum_map::{Enum, EnumArray, EnumMap};
use enumset::EnumSet;
use lazy_static::lazy_static;
use smallvec::SmallVec;

use serde::{Deserialize, Serialize};

use crate::cards::ids::*;

use crate::tcg_model::enums::{Dice, Element};
use crate::types::dice_counter::DiceCounter;
use crate::types::game_state::*;

#[cfg(HASH128)]
pub type HashValue = u128;

#[cfg(not(HASH128))]
pub type HashValue = u64;

/// Module containing mutation methods for `GameState`, `PlayerState` and `CharState`
/// while maintain the Zobrist hash.
///
/// # Hash coherence
/// The `GameState` is hash coherent if and only if the incrementally-updated hash is
/// identical to the recomputed hash.
///
/// ```text
/// let incremental_hash = game_state.zobrist_hash();
/// let recomputed_hash = { game_state.rehash(); game_state.zobrist_hash() };
/// assert_eq!(incremental_hash, recomputed_hash);
/// ```
pub(crate) mod game_state_mutation;

pub(crate) mod hash_provider;

pub(crate) use hash_provider::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ZobristHasher(pub HashValue);

impl ZobristHasher {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn combine(&mut self, Self(v): Self) {
        self.0 ^= v
    }

    #[inline]
    pub fn hash(&mut self, v: HashValue) {
        self.0 ^= v
    }

    #[inline]
    pub fn finish(self) -> HashValue {
        self.0
    }
}

impl Phase {
    const VALUES: [Phase; 16] = [
        Self::RollPhase {
            first_active_player: PlayerId::PlayerFirst,
            roll_phase_state: RollPhaseState::Start,
        },
        Self::RollPhase {
            first_active_player: PlayerId::PlayerSecond,
            roll_phase_state: RollPhaseState::Start,
        },
        Self::RollPhase {
            first_active_player: PlayerId::PlayerFirst,
            roll_phase_state: RollPhaseState::Drawing,
        },
        Self::RollPhase {
            first_active_player: PlayerId::PlayerSecond,
            roll_phase_state: RollPhaseState::Drawing,
        },
        Self::RollPhase {
            first_active_player: PlayerId::PlayerFirst,
            roll_phase_state: RollPhaseState::Rolling,
        },
        Self::RollPhase {
            first_active_player: PlayerId::PlayerSecond,
            roll_phase_state: RollPhaseState::Rolling,
        },
        Self::ActionPhase {
            first_end_round: None,
            active_player: PlayerId::PlayerFirst,
        },
        Self::ActionPhase {
            first_end_round: None,
            active_player: PlayerId::PlayerSecond,
        },
        Self::ActionPhase {
            first_end_round: Some(PlayerId::PlayerFirst),
            active_player: PlayerId::PlayerFirst,
        },
        Self::ActionPhase {
            first_end_round: Some(PlayerId::PlayerFirst),
            active_player: PlayerId::PlayerSecond,
        },
        Self::ActionPhase {
            first_end_round: Some(PlayerId::PlayerSecond),
            active_player: PlayerId::PlayerFirst,
        },
        Self::ActionPhase {
            first_end_round: Some(PlayerId::PlayerSecond),
            active_player: PlayerId::PlayerSecond,
        },
        Self::EndPhase {
            next_first_active_player: PlayerId::PlayerFirst,
        },
        Self::EndPhase {
            next_first_active_player: PlayerId::PlayerSecond,
        },
        Self::WinnerDecided {
            winner: PlayerId::PlayerFirst,
        },
        Self::WinnerDecided {
            winner: PlayerId::PlayerSecond,
        },
    ];

    #[inline]
    fn to_index(self) -> usize {
        match self {
            Self::RollPhase {
                first_active_player: PlayerId::PlayerFirst,
                roll_phase_state: RollPhaseState::Start,
            } => 0,
            Self::RollPhase {
                first_active_player: PlayerId::PlayerSecond,
                roll_phase_state: RollPhaseState::Start,
            } => 1,
            Self::RollPhase {
                first_active_player: PlayerId::PlayerFirst,
                roll_phase_state: RollPhaseState::Drawing,
            } => 2,
            Self::RollPhase {
                first_active_player: PlayerId::PlayerSecond,
                roll_phase_state: RollPhaseState::Drawing,
            } => 3,
            Self::RollPhase {
                first_active_player: PlayerId::PlayerFirst,
                roll_phase_state: RollPhaseState::Rolling,
            } => 4,
            Self::RollPhase {
                first_active_player: PlayerId::PlayerSecond,
                roll_phase_state: RollPhaseState::Rolling,
            } => 5,
            Self::ActionPhase {
                first_end_round: None,
                active_player: PlayerId::PlayerFirst,
            } => 6,
            Self::ActionPhase {
                first_end_round: None,
                active_player: PlayerId::PlayerSecond,
            } => 7,
            Self::ActionPhase {
                first_end_round: Some(PlayerId::PlayerFirst),
                active_player: PlayerId::PlayerFirst,
            } => 8,
            Self::ActionPhase {
                first_end_round: Some(PlayerId::PlayerFirst),
                active_player: PlayerId::PlayerSecond,
            } => 9,
            Self::ActionPhase {
                first_end_round: Some(PlayerId::PlayerSecond),
                active_player: PlayerId::PlayerFirst,
            } => 10,
            Self::ActionPhase {
                first_end_round: Some(PlayerId::PlayerSecond),
                active_player: PlayerId::PlayerSecond,
            } => 11,
            Self::EndPhase {
                next_first_active_player: PlayerId::PlayerFirst,
            } => 12,
            Self::EndPhase {
                next_first_active_player: PlayerId::PlayerSecond,
            } => 13,
            Self::WinnerDecided {
                winner: PlayerId::PlayerFirst,
            } => 14,
            Self::WinnerDecided {
                winner: PlayerId::PlayerSecond,
            } => 15,
        }
    }
}

impl Enum for Phase {
    const LENGTH: usize = 16;

    fn from_usize(value: usize) -> Self {
        Self::VALUES[value]
    }

    fn into_usize(self) -> usize {
        Self::VALUES
            .iter()
            .copied()
            .enumerate()
            .find(|(_, x)| *x == self)
            .unwrap()
            .0
    }
}

impl<V> EnumArray<V> for Phase {
    type Array = [V; 16];
}

impl GameState {
    /// Recompute the entire Zobrist hash from beginning without updating
    pub fn zobrist_hash_full_recompute(&self, h: &mut ZobristHasher) {
        self.incremental_zobrist_hash(h);
        self.non_incremental_zobrist_hash(h);
    }

    /// Get the current Zobrist hash that is computed incrementally.
    #[inline]
    pub fn zobrist_hash(&self) -> HashValue {
        self._hash.finish()
    }

    /// Recompute the incremental portion of the Zobrist hash without updating `self._hash`.
    pub fn incremental_zobrist_hash(&self, h: &mut ZobristHasher) {
        h.hash(HASH_PROVIDER.phase(self.phase));
        self.players.0.incremental_zobrist_hash(h, PlayerId::PlayerFirst);
        self.players.1.incremental_zobrist_hash(h, PlayerId::PlayerSecond);
    }

    /// Compute the non-incremental portion of the Zobrist hash
    pub fn non_incremental_zobrist_hash(&self, h: &mut ZobristHasher) {
        self.players.0.non_incremental_zobrist_hash(h, PlayerId::PlayerFirst);
        self.players.1.non_incremental_zobrist_hash(h, PlayerId::PlayerSecond);
        self.pending_cmds_hash(h);
    }

    #[inline]
    pub fn pending_cmds_hash(&self, h: &mut ZobristHasher) {
        if let Some(pc) = &self.pending_cmds {
            match pc.suspended_state {
                SuspendedState::PostDeathSwitch {
                    player_id,
                    character_statuses_to_shift: cs,
                } => {
                    h.hash(HASH_PROVIDER.post_death_switch(player_id));
                    for (index, status_entry) in cs.iter().flatten().enumerate() {
                        status_entry.zobrist_hash(h, index, player_id);
                    }
                }
                SuspendedState::NondetRequest(req) => h.hash(HASH_PROVIDER.nondet_request(req)),
            }
        }
    }

    /// Re-compute the entire Zobrist hash.
    /// Call this function after mutating the game state manually (i.e. without using methods in `game_state_mutation`).
    #[inline]
    pub fn rehash(&mut self) {
        let mut ih = ZobristHasher::new();
        self.incremental_zobrist_hash(&mut ih);
        self._incremental_hash = ih;
        self.non_incremental_zobrist_hash(&mut ih);
        self._hash = ih;
    }

    #[inline]
    pub fn update_hash(&mut self) {
        let mut h = self._incremental_hash;
        self.non_incremental_zobrist_hash(&mut h);
        self._hash = h;
    }
}

impl PlayerState {
    #[inline]
    pub(crate) fn tally_hand(hand: &SmallVec<[CardId; 4]>) -> SmallVec<[(CardId, u8); 4]> {
        let mut v = SmallVec::<[_; 4]>::new();
        for card_id in hand {
            v.push((*card_id, 0_u8));
        }
        for card_id in hand {
            v.iter_mut().find(|(c, _)| *card_id == *c).unwrap().1 += 1;
        }
        v
    }

    #[inline]
    pub(crate) fn hash_hand(hand: &SmallVec<[CardId; 4]>, h: &mut ZobristHasher, player_id: PlayerId) {
        for (card_id, count) in Self::tally_hand(hand) {
            h.hash(HASH_PROVIDER.hand(player_id, card_id, count));
        }
    }

    #[inline]
    pub fn zobrist_hash_for_hand(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        Self::hash_hand(&self.hand, h, player_id)
    }

    #[inline]
    pub fn zobrist_hash_for_dice(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        Self::dice_hash(h, player_id, &self.dice);
    }

    #[inline]
    pub fn zobrist_hash_for_flags(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
    }

    pub fn zobrist_hash_for_char_states(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        for (i, cs) in self.char_states.iter().enumerate() {
            cs.zobrist_hash(h, player_id, i as u8);
        }
    }

    pub fn zobrist_hash_full_recompute(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        self.incremental_zobrist_hash(h, player_id);
        self.non_incremental_zobrist_hash(h, player_id);
    }

    pub fn incremental_zobrist_hash(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        h.hash(HASH_PROVIDER.active_char_index(player_id, self.active_char_index));
        self.status_collection.zobrist_hash(h, player_id);
        self.zobrist_hash_for_flags(h, player_id);
        self.zobrist_hash_for_dice(h, player_id);
        self.zobrist_hash_for_hand(h, player_id);
        self.zobrist_hash_for_char_states(h, player_id);
    }

    #[inline]
    pub fn non_incremental_zobrist_hash(&self, _h: &mut ZobristHasher, _player_id: PlayerId) {
        // empty
    }
}

impl CharState {
    #[inline]
    pub fn zobrist_hash(&self, h: &mut ZobristHasher, player_id: PlayerId, char_idx: u8) {
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.get_hp()));
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, self.get_energy()));
        h.hash(HASH_PROVIDER.applied_elements(player_id, char_idx, self.applied));
        h.hash(HASH_PROVIDER.char_flags(player_id, char_idx, self.flags));
    }
}

impl StatusEntry {
    #[inline]
    pub fn zobrist_hash(&self, h: &mut ZobristHasher, index: usize, player_id: PlayerId) {
        let (a, b) = (
            self.state.get_usages(),
            self.state.get_counter() + if self.state.can_use_once_per_round() { 8 } else { 0 },
        );
        let hv: HashValue = match self.key {
            StatusKey::Character(char_idx, status_id) | StatusKey::Equipment(char_idx, _, status_id) => {
                HASH_PROVIDER.character_status(player_id, char_idx, status_id, a, b)
            }
            StatusKey::Team(status_id) => HASH_PROVIDER.team_status(player_id, status_id, a, b),
            StatusKey::Summon(summon_id) => HASH_PROVIDER.summon_status(player_id, summon_id, a, b),
            StatusKey::Support(slot, support_id) => HASH_PROVIDER.support_status(player_id, slot, support_id, a, b),
        };
        h.hash(HashProvider::with_index(hv, index));
    }
}

impl StatusCollection {
    #[inline]
    pub fn zobrist_hash(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        for (index, s) in self._status_entries.iter().enumerate() {
            s.zobrist_hash(h, index, player_id)
        }
    }
}
