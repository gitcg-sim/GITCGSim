use crate::data_structures::capped_list::CappedLengthList8;
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
use crate::std_subset::fmt::Debug;
use crate::std_subset::hash::Hash;

use enum_map::{Enum, EnumArray, EnumMap};
use enumset::EnumSet;
use lazy_static::lazy_static;

use crate::cards::ids::*;

use crate::tcg_model::{Dice, Element};
use crate::types::dice_counter::DiceCounter;
use crate::types::game_state::*;

#[cfg(feature = "hash128")]
pub type HashValue = u128;

#[cfg(not(feature = "hash128"))]
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
mod game_state_mutation;

pub(crate) use game_state_mutation::PlayerHashContext;

mod hash_provider;

pub(crate) use hash_provider::{HashProvider, HASH_PROVIDER};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    const VALUES: [Phase; 20] = [
        Self::Drawing {
            first_active_player: PlayerId::PlayerFirst,
        },
        Self::Drawing {
            first_active_player: PlayerId::PlayerSecond,
        },
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
        Self::SelectStartingCharacter {
            state: SelectStartingCharacterState::Start {
                to_select: PlayerId::PlayerFirst,
            },
        },
        Self::SelectStartingCharacter {
            state: SelectStartingCharacterState::Start {
                to_select: PlayerId::PlayerSecond,
            },
        },
        Self::SelectStartingCharacter {
            state: SelectStartingCharacterState::FirstSelected {
                to_select: PlayerId::PlayerFirst,
            },
        },
        Self::SelectStartingCharacter {
            state: SelectStartingCharacterState::FirstSelected {
                to_select: PlayerId::PlayerSecond,
            },
        },
    ];
    const COUNT: usize = Self::VALUES.len();

    #[inline]
    const fn to_index(self) -> usize {
        match self {
            Self::Drawing {
                first_active_player: PlayerId::PlayerFirst,
            } => 0,
            Self::Drawing {
                first_active_player: PlayerId::PlayerSecond,
            } => 1,
            Self::RollPhase {
                first_active_player: PlayerId::PlayerFirst,
                roll_phase_state: RollPhaseState::Start,
            } => 2,
            Self::RollPhase {
                first_active_player: PlayerId::PlayerSecond,
                roll_phase_state: RollPhaseState::Start,
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
            Self::SelectStartingCharacter {
                state:
                    SelectStartingCharacterState::Start {
                        to_select: PlayerId::PlayerFirst,
                    },
            } => 16,
            Self::SelectStartingCharacter {
                state:
                    SelectStartingCharacterState::Start {
                        to_select: PlayerId::PlayerSecond,
                    },
            } => 17,
            Self::SelectStartingCharacter {
                state:
                    SelectStartingCharacterState::FirstSelected {
                        to_select: PlayerId::PlayerFirst,
                    },
            } => 18,
            Self::SelectStartingCharacter {
                state:
                    SelectStartingCharacterState::FirstSelected {
                        to_select: PlayerId::PlayerSecond,
                    },
            } => 19,
        }
    }
}

impl Enum for Phase {
    const LENGTH: usize = Phase::COUNT;

    fn from_usize(value: usize) -> Self {
        Self::VALUES[value]
    }

    fn into_usize(self) -> usize {
        Self::VALUES
            .iter()
            .copied()
            .enumerate()
            .find(|(_, x)| *x == self)
            .expect("Phase::into_usize: must have result")
            .0
    }
}

impl<V> EnumArray<V> for Phase {
    type Array = [V; Phase::COUNT];
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
        self.players
            .for_each(|player_id, player| player.incremental_zobrist_hash(h, player_id));
        self.status_collections
            .for_each(|player_id, sc| sc.zobrist_hash(h, player_id));
    }

    /// Compute the non-incremental portion of the Zobrist hash
    pub fn non_incremental_zobrist_hash(&self, h: &mut ZobristHasher) {
        self.players
            .for_each(|player_id, player| player.non_incremental_zobrist_hash(h, player_id));
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
    pub(crate) fn tally_hand(
        hand: &CappedLengthList8<CardId, { Self::HAND_SIZE_LIMIT }>,
    ) -> heapless::Vec<(CardId, u8), { Self::HAND_SIZE_LIMIT }> {
        let mut v = heapless::Vec::<_, { Self::HAND_SIZE_LIMIT }>::default();
        for &card_id in hand.iter() {
            let _ignored = v.push((card_id, 0_u8));
        }
        for &card_id in hand.iter() {
            v.iter_mut()
                .find(|(c, _)| card_id == *c)
                .expect("tally_hand: card_id does not exist")
                .1 += 1;
        }
        v
    }

    #[inline]
    pub(crate) fn hash_hand(
        hand: &CappedLengthList8<CardId, { Self::HAND_SIZE_LIMIT }>,
        h: &mut ZobristHasher,
        player_id: PlayerId,
    ) {
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
        for (i, cs) in self.char_states.iter_all().enumerate() {
            cs.zobrist_hash(h, player_id, i as u8);
        }
    }

    pub fn zobrist_hash_full_recompute(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        self.incremental_zobrist_hash(h, player_id);
        self.non_incremental_zobrist_hash(h, player_id);
    }

    pub fn incremental_zobrist_hash(&self, h: &mut ZobristHasher, player_id: PlayerId) {
        h.hash(HASH_PROVIDER.active_char_idx(player_id, self.active_char_idx));
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
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.hp()));
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, self.energy()));
        h.hash(HASH_PROVIDER.applied_elements(player_id, char_idx, self.applied));
        h.hash(HASH_PROVIDER.char_flags(player_id, char_idx, self.flags));
        h.hash(HASH_PROVIDER.total_dmg_taken(player_id, char_idx, self.total_dmg_taken));
    }
}

impl StatusEntry {
    #[inline]
    pub fn zobrist_hash(&self, h: &mut ZobristHasher, index: usize, player_id: PlayerId) {
        let (a, b) = (
            self.state.usages(),
            self.state.counter() + if self.state.can_use_once_per_round() { 8 } else { 0 },
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
        for (index, s) in self.status_entries.iter().enumerate() {
            s.zobrist_hash(h, index, player_id)
        }
    }
}

pub trait ZobristHashable {
    fn zobrist_hash(&self) -> HashValue;
}

#[cfg(test)]
#[test]
fn to_index_roundtrip() {
    for (i, phase) in Phase::VALUES.iter().copied().enumerate() {
        assert_eq!(phase, Phase::VALUES[phase.to_index()]);
        assert_eq!(i, phase.to_index());
    }
}
