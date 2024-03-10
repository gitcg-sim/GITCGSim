use super::*;

use crate::{phc, status_impls::prelude::Cost, types::ElementSet, zobrist_hash::PlayerHashContext};

impl CharState {
    #[inline]
    pub fn add_energy(&mut self, e: u8) {
        let max_energy = self.char_id.char_card().max_energy;
        let te = self.energy() + e;
        self.set_energy(if te > max_energy { max_energy } else { te });
    }

    #[inline]
    pub fn heal(&mut self, e: u8) {
        let max_health = self.char_id.char_card().max_health;
        let th = self.hp() + e;
        self.set_hp(if th > max_health { max_health } else { th });
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        self.hp() == 0
    }

    #[inline]
    pub fn can_pay_energy_cost(&self, cost: &Cost) -> bool {
        if cost.energy_cost > 0 && self.energy() < cost.energy_cost {
            return false;
        }
        true
    }
}

impl PlayerState {
    #[inline]
    pub fn character_card(&self, char_idx: u8) -> &'static CharCard {
        self.char_states[char_idx].char_id.char_card()
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
    pub fn active_character(&self) -> &CharState {
        &self.char_states[self.active_char_idx]
    }

    #[inline]
    pub fn active_character_mut(&mut self) -> &mut CharState {
        &mut self.char_states[self.active_char_idx]
    }

    pub fn switch_character_hashed(&mut self, c: PlayerHashContext, char_idx: u8) -> bool {
        if !self.is_valid_char_idx(char_idx) || char_idx == self.active_char_idx {
            false
        } else {
            self.set_active_char_idx(c, char_idx);
            self.active_char_idx = char_idx;
            true
        }
    }

    pub fn update_incremental_element_priority(&mut self) {
        for char_idx in 0..self.char_states.len() {
            self.update_incremental_element_priority_for_char(char_idx);
        }
    }

    fn update_incremental_element_priority_for_char(&mut self, char_idx: u8) {
        if self.char_states[char_idx].is_invalid() {
            return;
        }
        let ep = self.element_priority_switch(char_idx);
        self.char_states[char_idx].set_incremental_element_priority(ep);
    }

    #[inline]
    pub fn element_priority_for_cost_type(&self, cost_type: CostType) -> ElementPriority {
        match cost_type {
            CostType::Switching { dst_char_idx } => self.element_priority_switch(dst_char_idx),
            CostType::Card(..) | CostType::Skill(..) => self.element_priority(),
        }
    }

    #[inline]
    pub fn element_priority(&self) -> ElementPriority {
        let char_idx = self.active_char_idx;
        if let Some(ep) = self.char_states[char_idx].incremental_element_priority() {
            ep
        } else {
            self.element_priority_switch(char_idx)
        }
    }

    pub fn element_priority_switch(&self, dst_char_idx: u8) -> ElementPriority {
        let mut important_elems = ElementSet::default();
        let mut active_elem = Default::default();
        for (i, c) in self.char_states.enumerate_valid() {
            let e = c.char_id.char_card().elem;
            important_elems.insert(e);
            if i == dst_char_idx {
                active_elem = Some(e);
            }
        }
        ElementPriority::new(important_elems, active_elem)
    }

    pub fn dice_distribution(&self, status_collection: &StatusCollection) -> DiceDistribution {
        let mut dist = DiceDistribution::new(8, 1, self.element_priority(), Default::default());
        self.update_dice_distribution(status_collection, &mut dist);
        dist
    }

    /// While there is an off-element dice and a card on hand:
    /// Remove the dice and card and add an Omni dice
    pub fn pseudo_elemental_tuning(&mut self, (h, player_id): PlayerHashContext) {
        if self.hand.is_empty() {
            return;
        }

        let off_elems = self.element_priority().off_elems();

        let mut dice = self.dice;
        for elem in off_elems {
            while dice[Dice::Elem(elem)] > 0 && !self.hand.is_empty() {
                // TODO pick CardId::BlankCard only
                self.remove_card_from_hand_by_index((h, player_id), 0);
                dice.sub_single(Dice::Elem(elem), 1);
                dice.add_single(Dice::Omni, 1);
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
            dice.sub_single(Dice::Elem(elem), 1);
            dice.add_single(Dice::Omni, 1);
            break;
        }
        self.set_dice((h, player_id), &dice);
    }

    pub fn status_spec_modifiers(
        &self,
        status_collection: &StatusCollection,
        key: StatusKey,
    ) -> Option<StatusSpecModifier> {
        let mut modifiers = StatusSpecModifier::default();
        let mut changed = false;
        let active_char = self.active_character();
        if active_char.has_talent_equipped() {
            let status = key.status();
            if let Some((target_char_id, count)) = status.talent_usages_increase {
                if target_char_id == active_char.char_id {
                    modifiers.push(key, count);
                    changed = true;
                }
            }
        }

        if status_collection.responds_to.contains(RespondsTo::UpdateStatusSpec) {
            status_collection.consume_statuses_immutable(
                CharIdxSelector::One(self.active_char_idx),
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
    pub fn active_character_has_talent_equipped(&self) -> bool {
        self.active_character().has_talent_equipped()
    }
}

impl StatusCollection {
    pub fn ensure_unequipped(&mut self, char_idx: u8, slot: EquipSlot) {
        self.ensure_weapon_unequipped(char_idx, slot)
    }

    pub fn add_support_to_slot_replacing_existing(&mut self, slot: SupportSlot, support_id: SupportId) {
        let status_spec = support_id.status();
        if status_spec.attach_mode != StatusAttachMode::Support {
            panic!("add_support_to_slot_replacing_existing: wrong StatusAttachMode");
        }
        let f = StatusKeyFilter::Support(slot);
        self.status_entries.retain(|e| !f.matches(e.key));
        // Support usages cannot be buffed
        self.apply_or_refresh_status(StatusKey::Support(slot, support_id), status_spec, &None);
    }

    #[inline]
    pub fn is_preparing_skill(&self, active_char_idx: u8) -> bool {
        self.find_preparing_skill(active_char_idx).is_some()
    }
}

impl<P: GameStateParams> GameState<P> {
    #[inline]
    pub fn player(&self, player_id: PlayerId) -> &PlayerState {
        self.players.get(player_id)
    }

    #[inline]
    pub fn player_mut(&mut self, player_id: PlayerId) -> &mut PlayerState {
        self.players.get_mut(player_id)
    }

    #[inline]
    pub fn status_collection(&self, player_id: PlayerId) -> &StatusCollection {
        self.status_collections.get(player_id)
    }

    #[inline]
    pub fn active_player(&self) -> Option<&PlayerState> {
        self.phase.active_player().map(|p| match p {
            PlayerId::PlayerFirst => &self.players.0,
            PlayerId::PlayerSecond => &self.players.1,
        })
    }

    #[inline]
    pub fn active_player_mut(&mut self) -> Option<&mut PlayerState> {
        self.phase.active_player().map(|p| match p {
            PlayerId::PlayerFirst => &mut self.players.0,
            PlayerId::PlayerSecond => &mut self.players.1,
        })
    }

    #[inline]
    pub fn active_character(&self) -> Option<&CharState> {
        self.active_player().map(|x| x.active_character())
    }

    #[inline]
    pub fn active_character_id(&self) -> Option<CharId> {
        self.active_player().map(|x| x.active_character().char_id)
    }

    pub fn convert_to_tactical_search(&mut self) {
        for player in [&mut self.players.0, &mut self.players.1] {
            player.set_tactical(phc!(self, PlayerId::PlayerFirst), true);
            player.pseudo_elemental_tuning(phc!(self, PlayerId::PlayerFirst));
        }
        self.rehash();
    }

    pub fn perform_pseudo_elemental_tuning(&mut self, player_id: PlayerId) {
        match player_id {
            PlayerId::PlayerFirst => self.players.0.pseudo_elemental_tuning(phc!(self, player_id)),
            PlayerId::PlayerSecond => self.players.1.pseudo_elemental_tuning(phc!(self, player_id)),
        }
    }
}
