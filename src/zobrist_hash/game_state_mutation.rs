use crate::types::ElementSet;

use super::*;

impl GameState {
    #[inline]
    pub fn set_phase(&mut self, phase: Phase) {
        self._incremental_hash.hash(HASH_PROVIDER.phase(self.phase));
        self.phase = phase;
        self._incremental_hash.hash(HASH_PROVIDER.phase(self.phase));
    }

    #[inline]
    pub(crate) fn set_tactical(&mut self, tactical: bool) {
        self._incremental_hash.hash(HASH_PROVIDER.tactical(tactical));
        self.tactical = tactical;
        self._incremental_hash.hash(HASH_PROVIDER.tactical(tactical));
    }
}

pub type PlayerHashContext<'a> = (&'a mut ZobristHasher, PlayerId);

#[macro_export]
macro_rules! phc {
    ($self_game_state: expr, $player_id: expr) => {
        (&mut ($self_game_state)._incremental_hash, $player_id)
    };
}

// TODO refactor status collection operations into their own setters

impl PlayerState {
    #[inline]
    pub fn add_card_to_hand(&mut self, (h, player_id): PlayerHashContext, card_id: CardId) {
        let count = self.hand.iter().copied().filter(|c| *c == card_id).count() as u8;
        h.hash(HASH_PROVIDER.hand(player_id, card_id, count));
        self.hand.push(card_id);
        h.hash(HASH_PROVIDER.hand(player_id, card_id, count + 1));
    }

    #[inline]
    pub fn remove_card_from_hand_by_index(&mut self, (h, player_id): PlayerHashContext, i: usize) {
        let card_id = self.hand[i];
        let count = self.hand.iter().copied().filter(|c| *c == card_id).count() as u8;
        h.hash(HASH_PROVIDER.hand(player_id, card_id, count));
        self.hand.remove(i);
        h.hash(HASH_PROVIDER.hand(player_id, card_id, count - 1));
    }

    #[inline]
    pub fn try_remove_card_from_hand(&mut self, c: PlayerHashContext, card_id: CardId) -> bool {
        let Some(i) = self.hand.iter().position(|&x| x == card_id) else {
            return false
        };
        self.remove_card_from_hand_by_index(c, i);
        true
    }

    #[inline]
    pub fn update_active_char_index(&mut self, (h, player_id): PlayerHashContext, active_char_index: u8) {
        h.hash(HASH_PROVIDER.active_char_index(player_id, self.active_char_index));
        self.active_char_index = active_char_index;
        h.hash(HASH_PROVIDER.active_char_index(player_id, active_char_index));
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
        self.check_for_charged_attack();
    }

    #[inline]
    pub fn add_dice(&mut self, (h, player_id): PlayerHashContext, dice: &DiceCounter) {
        Self::dice_hash(h, player_id, &self.dice);
        self.dice.add_in_place(dice);
        Self::dice_hash(h, player_id, &self.dice);
        self.check_for_charged_attack();
    }

    #[inline]
    pub fn subtract_dice(&mut self, (h, player_id): PlayerHashContext, dice: &DiceCounter) {
        Self::dice_hash(h, player_id, &self.dice);
        self.dice.subtract_in_place(dice);
        Self::dice_hash(h, player_id, &self.dice);
        self.check_for_charged_attack();
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
        h.hash(HASH_PROVIDER.dice(player_id, add, self.dice[add]));
        h.hash(HASH_PROVIDER.dice(player_id, remove, self.dice[remove]));
        self.dice[add] += 1;
        self.dice[remove] -= 1;
        h.hash(HASH_PROVIDER.dice(player_id, add, self.dice[add]));
        h.hash(HASH_PROVIDER.dice(player_id, remove, self.dice[remove]));
        self.check_for_charged_attack();
    }

    #[inline]
    pub fn clear_flags_for_end_of_turn(&mut self) {
        self.flags.remove_all(PlayerFlags::END_OF_TURN_CLEAR);
    }

    #[inline]
    pub(crate) fn dice_hash(h: &mut ZobristHasher, player_id: PlayerId, dice: &DiceCounter) {
        h.hash(HASH_PROVIDER.dice(player_id, Dice::Omni, dice.omni));
        for e in Element::VALUES {
            h.hash(HASH_PROVIDER.dice(player_id, Dice::Elem(e), dice.elem[e.to_index()]));
        }
    }
}

pub type CharacterHashContext<'a> = (&'a mut ZobristHasher, PlayerId, u8);

#[macro_export]
macro_rules! chc {
    ($self_game_state: expr, $player_id: expr, $char_idx: expr) => {
        (&mut ($self_game_state)._incremental_hash, $player_id, $char_idx)
    };
}

impl CharState {
    #[inline]
    pub fn set_hp_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, hp: u8) {
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.get_hp()));
        self.set_hp(hp);
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, hp));
    }

    #[inline]
    pub fn reduce_hp_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, dmg_value: u8) {
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.get_hp()));
        self.reduce_hp(dmg_value);
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.get_hp()));
    }

    #[inline]
    pub fn heal_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, value: u8) {
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.get_hp()));
        self.heal(value);
        h.hash(HASH_PROVIDER.hp(player_id, char_idx, self.get_hp()));
    }

    #[inline]
    pub fn set_energy_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, energy: u8) {
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, self.get_energy()));
        self.set_energy(energy);
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, energy));
    }

    #[inline]
    pub fn add_energy_hashed(&mut self, (h, player_id, char_idx): CharacterHashContext, energy: u8) {
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, self.get_energy()));
        self.add_energy(energy);
        h.hash(HASH_PROVIDER.energy(player_id, char_idx, self.get_energy()));
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
}
