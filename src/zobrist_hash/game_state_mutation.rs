use crate::types::ElementSet;

use super::*;

impl GameState {
    #[inline]
    pub fn set_phase(&mut self, phase: Phase) {
        self._incremental_hash.hash(HASH_PROVIDER.phase(self.phase));
        self.phase = phase;
        self._incremental_hash.hash(HASH_PROVIDER.phase(self.phase));
    }
}

/// Handle used to update the Zobrist hash, focusing on a specific player.
pub type PlayerHashContext<'a> = (&'a mut ZobristHasher, PlayerId);

/// Constructs a new `PlayerHashContext` given mutable game state and player ID.
/// Avoids borrowing the entire `&mut GameState`.
#[macro_export]
#[doc(hidden)]
macro_rules! phc {
    ($self_game_state: expr, $player_id: expr) => {
        (&mut ($self_game_state)._incremental_hash, $player_id)
    };
}

impl PlayerState {
    #[inline]
    pub fn add_card_to_hand(&mut self, (h, player_id): PlayerHashContext, card_id: CardId) {
        let count = self.hand.iter().copied().filter(|c| *c == card_id).count() as u8;
        if self.hand.push(card_id).is_err() {
            return;
        };
        h.hash(HASH_PROVIDER.hand(player_id, card_id, count));
        h.hash(HASH_PROVIDER.hand(player_id, card_id, count + 1));
    }

    #[inline]
    pub fn remove_card_from_hand_by_index(&mut self, (h, player_id): PlayerHashContext, i: u8) {
        let card_id = self.hand[i];
        let count = self.hand.iter().copied().filter(|c| *c == card_id).count() as u8;
        h.hash(HASH_PROVIDER.hand(player_id, card_id, count));
        self.hand.remove(i);
        h.hash(HASH_PROVIDER.hand(player_id, card_id, count - 1));
    }

    #[inline]
    pub fn try_remove_card_from_hand(&mut self, c: PlayerHashContext, card_id: CardId) -> bool {
        let Some(i) = self.hand.iter().position(|&x| x == card_id) else {
            return false;
        };
        self.remove_card_from_hand_by_index(c, i as u8);
        true
    }

    #[inline]
    pub fn set_active_char_idx(&mut self, (h, player_id): PlayerHashContext, active_char_idx: u8) {
        h.hash(HASH_PROVIDER.active_char_idx(player_id, self.active_char_idx));
        self.active_char_idx = active_char_idx;
        h.hash(HASH_PROVIDER.active_char_idx(player_id, active_char_idx));
    }

    #[inline]
    pub fn set_dice_after_paying_cast(&mut self, (h, player_id): PlayerHashContext, dice: &DiceCounter) {
        Self::dice_hash(h, player_id, &self.dice);
        self.dice = *dice;
        Self::dice_hash(h, player_id, &self.dice);
        // Does not check for charged attack here
    }

    #[inline]
    pub fn set_dice(&mut self, (h, player_id): PlayerHashContext, dice: &DiceCounter) {
        Self::dice_hash(h, player_id, &self.dice);
        self.dice = *dice;
        Self::dice_hash(h, player_id, &self.dice);
        self.check_for_charged_attack((h, player_id));
    }

    #[inline]
    pub fn add_single_dice(&mut self, (h, player_id): PlayerHashContext, dice: Dice, value: u8) {
        h.hash(HASH_PROVIDER.dice(player_id, dice, self.dice[dice]));
        self.dice.add_single(dice, value);
        h.hash(HASH_PROVIDER.dice(player_id, dice, self.dice[dice]));
        self.check_for_charged_attack((h, player_id));
    }

    #[inline]
    pub fn add_dice(&mut self, (h, player_id): PlayerHashContext, dice: &DiceCounter) {
        Self::dice_hash(h, player_id, &self.dice);
        self.dice.add_dice(dice);
        Self::dice_hash(h, player_id, &self.dice);
        self.check_for_charged_attack((h, player_id));
    }

    #[inline]
    pub fn subtract_dice(&mut self, (h, player_id): PlayerHashContext, dice: &DiceCounter) {
        Self::dice_hash(h, player_id, &self.dice);
        self.dice.subtract_dice(dice);
        Self::dice_hash(h, player_id, &self.dice);
        self.check_for_charged_attack((h, player_id));
    }

    #[inline]
    pub fn update_dice_for_elemental_tuning(
        &mut self,
        (h, player_id): PlayerHashContext,
        elem_to_remove: Element,
        elem_to_add: Element,
    ) {
        let add = Dice::Elem(elem_to_add);
        let remove = Dice::Elem(elem_to_remove);
        Self::dice_hash(h, player_id, &self.dice);
        self.dice.add_single(add, 1);
        self.dice.sub_single(remove, 1);
        Self::dice_hash(h, player_id, &self.dice);
        self.check_for_charged_attack((h, player_id));
    }

    #[inline]
    pub fn clear_flags_for_end_of_turn(&mut self, (h, player_id): PlayerHashContext) {
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
        self.flags.remove_all(PlayerFlag::END_OF_TURN_CLEAR);
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
    }

    #[inline]
    pub fn insert_flag(&mut self, (h, player_id): PlayerHashContext, flag: PlayerFlag) {
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
        self.flags.insert(flag);
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
    }

    #[inline]
    pub fn remove_flag(&mut self, (h, player_id): PlayerHashContext, flag: PlayerFlag) {
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
        self.flags.remove(flag);
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
    }

    #[inline]
    pub fn set_tactical(&mut self, (h, player_id): PlayerHashContext, tactical: bool) {
        if tactical {
            self.insert_flag((h, player_id), PlayerFlag::Tactical);
        } else {
            self.remove_flag((h, player_id), PlayerFlag::Tactical);
        }
    }

    #[inline]
    pub fn check_for_charged_attack(&mut self, (h, player_id): PlayerHashContext) {
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
        if self.dice.parity() == 0 {
            self.flags.insert(PlayerFlag::ChargedAttack);
        } else {
            self.flags.remove(PlayerFlag::ChargedAttack);
        }
        h.hash(HASH_PROVIDER.player_flags(player_id, self.flags));
    }

    #[inline]
    pub(crate) fn dice_hash(h: &mut ZobristHasher, player_id: PlayerId, dice: &DiceCounter) {
        h.hash(HASH_PROVIDER.dice(player_id, Dice::Omni, dice[Dice::Omni]));
        for e in Element::VALUES {
            h.hash(HASH_PROVIDER.dice(player_id, Dice::Elem(e), dice[Dice::Elem(e)]));
        }
    }
}

/// Handle used to update the Zobrist hash, focusing on a specific player's character.
pub type CharacterHashContext<'a> = (&'a mut ZobristHasher, PlayerId, u8);

/// Constructs a new `CharacterHashContext` given mutable game state, player ID and character index.
/// Avoids borrowing the entire `&mut GameState`.
#[macro_export]
#[doc(hidden)]
macro_rules! chc {
    ($self_game_state: expr, $player_id: expr, $char_idx: expr) => {
        (&mut ($self_game_state)._incremental_hash, $player_id, $char_idx)
    };
}

impl CharState {
    #[inline]
    pub fn set_hp_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, hp: u8) {
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.hp()));
        self.set_hp(hp);
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, hp));
        if self.hp() == 0 {
            self.on_death_hashed((h, player_id, char_idx));
        }
    }

    #[inline]
    pub fn reduce_hp_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, dmg_value: u8) {
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.hp()));
        self.reduce_hp(dmg_value);
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.hp()));
        if self.hp() == 0 {
            self.on_death_hashed((h, player_id, char_idx));
        } else {
            h.hash(HASH_PROVIDER.total_dmg_taken(player_id, char_idx, self.total_dmg_taken));
            self.add_dmg_taken(dmg_value);
            h.hash(HASH_PROVIDER.total_dmg_taken(player_id, char_idx, self.total_dmg_taken));
        }
    }

    #[inline]
    fn on_death_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext) {
        self.set_energy_hashed((h, player_id, char_idx), 0);
        self.set_flags_hashed((h, player_id, char_idx), Default::default());
        self.set_applied_elements_hashed((h, player_id, char_idx), Default::default());
        h.hash(HASH_PROVIDER.total_dmg_taken(player_id, char_idx, self.total_dmg_taken));
        self.total_dmg_taken = 0;
        self.clear_incremental_element_priority();
        h.hash(HASH_PROVIDER.total_dmg_taken(player_id, char_idx, self.total_dmg_taken));
    }

    #[inline]
    pub fn heal_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, value: u8) {
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.hp()));
        self.heal(value);
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.hp()));
    }

    #[inline]
    pub fn set_energy_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, energy: u8) {
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, self.energy()));
        self.set_energy(energy);
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, energy));
    }

    #[inline]
    pub fn add_energy_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, energy: u8) {
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, self.energy()));
        self.add_energy(energy);
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, self.energy()));
    }

    #[inline]
    pub fn set_applied_elements_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, applied: ElementSet) {
        h.hash(HASH_PROVIDER.applied_elements(player_id, char_idx, self.applied));
        self.applied = applied;
        h.hash(HASH_PROVIDER.applied_elements(player_id, char_idx, applied));
    }

    #[inline]
    pub fn set_flags_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, flags: EnumSet<CharFlag>) {
        h.hash(HASH_PROVIDER.char_flags(player_id, char_idx, self.flags));
        self.flags = flags;
        h.hash(HASH_PROVIDER.char_flags(player_id, char_idx, flags));
    }

    #[inline]
    pub fn insert_flag_hashed(&mut self, chc: CharacterHashContext, flag: CharFlag) {
        self.set_flags_hashed(chc, self.flags | flag);
    }

    #[inline]
    pub fn remove_flag_hashed(&mut self, chc: CharacterHashContext, flag: CharFlag) {
        self.set_flags_hashed(chc, self.flags - flag);
    }
}

impl crate::types::by_player::ByPlayer<StatusCollection> {
    pub fn mutate_hashed<F: FnOnce(&mut StatusCollection) -> R, R>(
        &mut self,
        (h, player_id): PlayerHashContext,
        f: F,
    ) -> R {
        let status_collection = self.get_mut(player_id);
        status_collection.zobrist_hash(h, player_id);
        let r = f(status_collection);
        status_collection.zobrist_hash(h, player_id);
        r
    }
}
