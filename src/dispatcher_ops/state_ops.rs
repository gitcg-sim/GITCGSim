use crate::cards::ids::lookup::GetStatus;
use crate::cards::ids::{GetCharCard, SupportId};

use crate::phc;
use crate::status_impls::prelude::Cost;
use crate::tcg_model::enums::*;
use crate::types::card_defs::CharCard;
use crate::types::dice_counter::distribution::DiceDistribution;
use crate::types::dice_counter::ElementPriority;
use crate::types::status_impl::{RespondsTo, StatusImpl};
use crate::types::StatusSpecModifier;
use crate::zobrist_hash::game_state_mutation::PlayerHashContext;
use crate::{cards::ids::CharId, types::game_state::*};

use super::update_dice_distribution;

impl CharState {
    #[inline]
    pub fn add_energy(&mut self, e: u8) {
        let max_energy = self.char_id.get_char_card().max_energy;
        let te = self.get_energy() + e;
        self.set_energy(if te > max_energy { max_energy } else { te });
    }

    #[inline]
    pub fn heal(&mut self, e: u8) {
        let max_health = self.char_id.get_char_card().max_health;
        let th = self.get_hp() + e;
        self.set_hp(if th > max_health { max_health } else { th });
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        self.get_hp() == 0
    }

    #[inline]
    pub fn can_pay_energy_cost(&self, cost: &Cost) -> bool {
        if cost.energy_cost > 0 && self.get_energy() < cost.energy_cost {
            return false;
        }
        true
    }
}

impl PlayerState {
    #[inline]
    pub fn get_character_card(&self, char_idx: u8) -> &'static CharCard {
        self.char_states[char_idx].char_id.get_char_card()
    }

    // TODO inline this
    /// Checks if char_idx refers to a valid and alive character.
    #[inline]
    pub fn is_valid_char_idx(&self, char_idx: u8) -> bool {
        self.char_states.is_valid_char_idx(char_idx)
    }

    #[inline]
    pub fn try_get_character(&self, char_idx: u8) -> Option<&CharState> {
        if self.is_valid_char_idx(char_idx) {
            Some(&self.char_states[char_idx])
        } else {
            None
        }
    }

    #[inline]
    pub fn try_get_character_mut(&mut self, char_idx: u8) -> Option<&mut CharState> {
        if self.is_valid_char_idx(char_idx) {
            Some(&mut self.char_states[char_idx])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_active_character(&self) -> &CharState {
        &self.char_states[self.active_char_idx]
    }

    #[inline]
    pub fn get_active_character_mut(&mut self) -> &mut CharState {
        &mut self.char_states[self.active_char_idx]
    }

    // Returns None for invalid index
    pub fn switch_character(&mut self, c: PlayerHashContext, char_idx: u8) -> bool {
        if !self.is_valid_char_idx(char_idx) || char_idx == self.active_char_idx {
            false
        } else {
            self.update_active_char_idx(c, char_idx);
            self.active_char_idx = char_idx;
            true
        }
    }

    // TODO need to take switch target into account
    pub fn get_element_priority(&self) -> ElementPriority {
        let mut ep = ElementPriority::default();
        for (i, c) in self.char_states.enumerate_valid() {
            let e = c.char_id.get_char_card().elem;
            ep.important_elems |= e;
            if i == self.active_char_idx {
                ep.active_elem = Some(e);
            }
        }
        ep
    }

    pub fn get_dice_distribution(&self) -> DiceDistribution {
        let mut dist = DiceDistribution::new(8, 1, self.get_element_priority(), Default::default());
        update_dice_distribution(self, &mut dist);
        dist
    }

    /// While there is an off-element dice and a card on hand:
    /// Remove the dice and card and add an Omni dice
    pub(crate) fn pseudo_elemental_tuning(&mut self, (h, player_id): PlayerHashContext) {
        if self.hand.is_empty() {
            return;
        }

        let off_elems = {
            let ep = self.get_element_priority();
            let mut es = ep.important_elems;
            if let Some(e) = ep.active_elem {
                es |= e;
            }
            !es
        };

        let mut dice = self.dice;
        for elem in off_elems {
            while dice[Dice::Elem(elem)] > 0 && !self.hand.is_empty() {
                // TODO pick CardId::BlankCard only
                self.remove_card_from_hand_by_index((h, player_id), 0);
                dice[Dice::Elem(elem)] -= 1;
                dice.omni += 1;
            }

            if self.hand.is_empty() {
                break;
            }
        }

        // 1 additional ET for free
        for elem in off_elems {
            if dice[Dice::Elem(elem)] == 0 {
                continue;
            }

            // 1 additional ET for free
            dice[Dice::Elem(elem)] -= 1;
            dice.omni += 1;
            break;
        }
        self.set_dice((h, player_id), &dice);
    }

    pub(crate) fn get_status_spec_modifiers(&self, key: StatusKey) -> Option<StatusSpecModifier> {
        let mut modifiers = StatusSpecModifier::default();
        let mut changed = false;
        let active_char = self.get_active_character();
        if active_char.has_talent_equipped() {
            let status = key.get_status();
            if let Some((target_char_id, count)) = status.talent_usages_increase {
                if target_char_id == active_char.char_id {
                    modifiers.push(key, count);
                    changed = true;
                }
            }
        }

        if self
            .status_collection
            .responds_to
            .contains(RespondsTo::UpdateStatusSpec)
        {
            self.status_collection.consume_statuses_immutable(
                CharacterIndexSelector::One(self.active_char_idx),
                |si| si.responds_to().contains(RespondsTo::UpdateStatusSpec),
                |_, _sk, si| {
                    if si.update_status_spec(&mut modifiers) {
                        Some(AppliedEffectResult::NoChange)
                    } else {
                        None
                    }
                },
            );
        }

        if changed {
            Some(modifiers)
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn active_character_has_talent_equipped(&self) -> bool {
        self.get_active_character().has_talent_equipped()
    }

    #[inline]
    pub fn is_preparing_skill(&self) -> bool {
        self.status_collection
            .find_preparing_skill(self.active_char_idx)
            .is_some()
    }
}

impl StatusCollection {
    pub fn ensure_unequipped(&mut self, char_idx: u8, slot: EquipSlot) {
        self.ensure_weapon_unequipped(char_idx, slot)
    }

    pub fn add_support_to_slot_replacing_existing(&mut self, slot: SupportSlot, support_id: SupportId) {
        let status_spec = support_id.get_status();
        if status_spec.attach_mode != StatusAttachMode::Support {
            panic!("add_support_to_slot_replacing_existing: wrong StatusAttachMode");
        }
        let f = StatusKeyFilter::Support(slot);
        self._status_entries.retain(|e| !f.matches(e.key));
        // Support usages cannot be buffed
        self.apply_or_refresh_status(StatusKey::Support(slot, support_id), status_spec, &None);
    }
}

impl GameState {
    #[inline]
    pub fn get_player(&self, player_id: PlayerId) -> &PlayerState {
        match player_id {
            PlayerId::PlayerFirst => &self.players.0,
            PlayerId::PlayerSecond => &self.players.1,
        }
    }

    #[inline]
    pub fn get_player_mut(&mut self, player_id: PlayerId) -> &mut PlayerState {
        match player_id {
            PlayerId::PlayerFirst => &mut self.players.0,
            PlayerId::PlayerSecond => &mut self.players.1,
        }
    }

    #[inline]
    pub fn get_active_player(&self) -> Option<&PlayerState> {
        self.phase.active_player().map(|p| match p {
            PlayerId::PlayerFirst => &self.players.0,
            PlayerId::PlayerSecond => &self.players.1,
        })
    }

    #[inline]
    pub fn get_active_player_mut(&mut self) -> Option<&mut PlayerState> {
        self.phase.active_player().map(|p| match p {
            PlayerId::PlayerFirst => &mut self.players.0,
            PlayerId::PlayerSecond => &mut self.players.1,
        })
    }

    #[inline]
    pub fn get_active_character(&self) -> Option<&CharState> {
        self.get_active_player().map(|x| x.get_active_character())
    }

    #[inline]
    pub fn get_active_character_id(&self) -> Option<CharId> {
        self.get_active_player().map(|x| x.get_active_character().char_id)
    }

    pub fn with_player(&self, player_id: PlayerId, player_state: &PlayerState) -> GameState {
        let mut gs1 = self.clone();
        match player_id {
            PlayerId::PlayerFirst => gs1.players.0 = player_state.clone(),
            PlayerId::PlayerSecond => gs1.players.1 = player_state.clone(),
        }
        gs1
    }

    pub(crate) fn convert_to_tactical_search(&mut self) {
        self.log.enabled = false;
        for player in [&mut self.players.0, &mut self.players.1] {
            player.set_tactical(phc!(self, PlayerId::PlayerFirst), true);
            player.pseudo_elemental_tuning(phc!(self, PlayerId::PlayerFirst));
        }
        self.rehash();
    }

    pub(crate) fn perform_pseudo_elemental_tuning(&mut self, player_id: PlayerId) {
        match player_id {
            PlayerId::PlayerFirst => self.players.0.pseudo_elemental_tuning(phc!(self, player_id)),
            PlayerId::PlayerSecond => self.players.1.pseudo_elemental_tuning(phc!(self, player_id)),
        }
    }
}
