use crate::cards::ids::*;
use crate::status_impls::*;
use crate::types::command::XEventMask;
use crate::types::status_impl::StatusImpl;
use crate::types::StatusSpecModifier;
use crate::{
    cards::ids::lookup::GetStatus,
    types::{card_defs::Status, command::EventId, enums::EquipSlot, game_state::*, status_impl::RespondsTo},
};
use enumset::enum_set;
use smallvec::SmallVec;

macro_rules! character_or_equipment {
    ($i: pat, $status_id: pat) => {
        StatusKey::Character($i, $status_id) | StatusKey::Equipment($i, _, $status_id)
    };
}

impl AppliedEffectState {
    #[inline]
    pub fn new(status: &'static Status) -> AppliedEffectState {
        let mut es =
            AppliedEffectState::from_fields(status.usages.unwrap_or(0), status.duration_rounds.unwrap_or(0), true);

        if let Some(cs) = &status.counter_spec {
            es.set_counter(cs.default_value);
        }

        es
    }

    pub fn should_be_eliminated(&self, status: &'static Status) -> bool {
        if status.manual_discard {
            return false;
        }
        if status.usages.is_some() && self.no_usages() {
            return true;
        }
        if status.duration_rounds.is_some() && self.no_duration() {
            return true;
        }
        false
    }

    /// Refresh the duration/usages for the status.
    fn refresh(&mut self, status: &'static Status) {
        if let Some(rounds) = status.duration_rounds {
            // Durations refresh to max
            self.set_duration(rounds);
        } else if let Some(usages) = status.usages {
            if let Some(ms) = status.max_stacks {
                let mut u = self.get_usages() + usages;
                if u > ms {
                    u = ms
                }
                self.set_usages(u)
            } else {
                self.set_usages(usages)
            }
        }
        // Counters are not refreshed
    }

    /// Returns: true if there are any changes
    #[inline]
    fn apply_change(&mut self, r: Option<AppliedEffectResult>) -> bool {
        match r {
            None => false,
            Some(AppliedEffectResult::NoChange) => true,
            Some(AppliedEffectResult::DeleteSelf) => true,
            Some(AppliedEffectResult::SetCounter(c)) => {
                self.set_counter(c);
                true
            }
            Some(AppliedEffectResult::SetCounterAndConsumeOncePerRound(c)) => {
                self.set_counter(c);
                self.set_once_per_round_false();
                true
            }
            Some(AppliedEffectResult::ConsumeUsage) => self.consume_usage(),
            Some(AppliedEffectResult::ConsumeUsages(n)) => {
                let u = self.get_usages();
                if self.get_usages() == 0 || n == 0 {
                    false
                } else if u >= n {
                    self.set_usages(u - n);
                    true
                } else {
                    self.set_usages(0);
                    true
                }
            }
            Some(AppliedEffectResult::ConsumeOncePerRound) => {
                self.set_once_per_round_false();
                true
            }
        }
    }

    #[inline]
    fn consume_usage(&mut self) -> bool {
        if !self.no_usages() {
            self.decrement_usages();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn end_of_turn(&mut self, status: &'static Status) {
        self.set_once_per_round_true();
        if status.duration_rounds.is_some() && !self.no_duration() {
            self.decrement_duration()
        }
        if let Some(cs) = &status.counter_spec {
            if cs.resets_at_turn_end {
                self.set_counter(cs.default_value)
            }
        }
    }

    #[inline]
    pub fn apply_change_and_check_retain(
        &mut self,
        found: &mut bool,
        res: Option<AppliedEffectResult>,
        status: &'static Status,
    ) -> bool {
        match res {
            Some(AppliedEffectResult::DeleteSelf) => {
                *found = true;
                false
            }
            _ => {
                *found |= self.apply_change(res);
                !self.should_be_eliminated(status)
            }
        }
    }
}

impl StatusCollection {
    #[inline]
    pub fn iter_entries(&self) -> std::slice::Iter<StatusEntry> {
        self._status_entries.iter()
    }

    #[inline]
    pub fn iter_entries_mut(&mut self) -> std::slice::IterMut<StatusEntry> {
        self._status_entries.iter_mut()
    }

    /// Precondition: For Char(index, status_id), the index must be valid
    #[inline]
    pub fn get(&self, key: StatusKey) -> Option<&AppliedEffectState> {
        self._status_entries.iter().find(|&e| e.key == key).map(|e| &e.state)
    }

    /// Precondition: For Char(index, status_id), the index must be valid
    #[inline]
    pub fn get_mut(&mut self, key: StatusKey) -> Option<&mut AppliedEffectState> {
        self._status_entries
            .iter_mut()
            .find(|e| e.key == key)
            .map(|e| &mut e.state)
    }

    pub fn delete(&mut self, key: StatusKey) {
        self._status_entries.retain(|s| s.key != key);
        self.refresh_responds_to();
    }

    #[inline]
    pub fn responds_to(&self, responds_to: RespondsTo) -> bool {
        self.responds_to.contains(responds_to)
    }

    #[inline]
    pub fn responds_to_trigger_event(&self, event_id: EventId) -> bool {
        self.responds_to_triggers.contains(event_id)
    }

    #[inline]
    pub fn responds_to_events(&self, mask: XEventMask) -> bool {
        !(self.responds_to_events & mask).is_empty()
    }

    #[inline]
    pub fn count(&self) -> usize {
        self._status_entries.len()
    }

    #[inline]
    pub fn support_count(&self) -> usize {
        self._status_entries
            .iter()
            .filter(|e| matches!(e.key, StatusKey::Support(..)))
            .count()
    }

    #[inline]
    pub fn summon_count(&self) -> usize {
        self._status_entries
            .iter()
            .filter(|e| matches!(e.key, StatusKey::Summon(..)))
            .count()
    }

    #[inline]
    pub fn team_status_count(&self) -> usize {
        self._status_entries
            .iter()
            .filter(|e| matches!(e.key, StatusKey::Team(..)))
            .count()
    }

    #[inline]
    pub fn status_count(&self) -> usize {
        self._status_entries
            .iter()
            .filter(|e| matches!(e.key, StatusKey::Character(..) | StatusKey::Team(..)))
            .count()
    }

    #[inline]
    pub fn equipment_count(&self, char_idx: u8) -> usize {
        self._status_entries
            .iter()
            .filter(|e| matches!(e.key, StatusKey::Equipment(i, _, _) if i == char_idx))
            .count()
    }

    #[inline]
    pub fn character_status_count(&self, char_idx: u8) -> usize {
        self._status_entries
            .iter()
            .filter(|e| matches!(e.key, StatusKey::Character(i, _) if i == char_idx))
            .count()
    }

    pub fn has_character_status(&self, char_idx: u8, status_id: StatusId) -> bool {
        self._status_entries.iter().any(|e| match e.key {
            StatusKey::Character(i, sid) => i == char_idx && sid == status_id,
            _ => false,
        })
    }

    pub fn has_team_status(&self, status_id: StatusId) -> bool {
        self._status_entries.iter().any(|e| match e.key {
            StatusKey::Team(sid) => sid == status_id,
            _ => false,
        })
    }

    pub fn has_summon(&self, summon_id: SummonId) -> bool {
        self._status_entries.iter().any(|e| match e.key {
            StatusKey::Summon(sid) => sid == summon_id,
            _ => false,
        })
    }

    pub fn find_equipment(&self, char_idx: u8, slot: EquipSlot) -> Option<&StatusEntry> {
        let f = StatusKeyFilter::Equipment(char_idx, slot);
        self._status_entries.iter().find(|e| f.matches(e.key))
    }

    pub fn find_support(&self, slot: SupportSlot) -> Option<&StatusEntry> {
        let f = StatusKeyFilter::Support(slot);
        self._status_entries.iter().find(|e| f.matches(e.key))
    }

    pub fn ensure_weapon_unequipped(&mut self, char_idx: u8, slot: EquipSlot) {
        let f = StatusKeyFilter::Equipment(char_idx, slot);
        self._status_entries.retain(|e| !f.matches(e.key))
    }

    pub fn find_equipment_mut(&mut self, char_idx: u8, slot: EquipSlot) -> Option<&mut StatusEntry> {
        let f = StatusKeyFilter::Equipment(char_idx, slot);
        self._status_entries.iter_mut().find(|e| f.matches(e.key))
    }

    pub fn team_statuses_vec(&self) -> Vec<&StatusEntry> {
        let f = StatusKeyFilter::Team;
        self._status_entries.iter().filter(|&s| f.matches(s.key)).collect()
    }

    pub fn equipment_statuses_vec(&self, char_idx: u8) -> Vec<(EquipSlot, StatusId, &AppliedEffectState)> {
        self._status_entries
            .iter()
            .filter_map(|s| {
                if let StatusKey::Equipment(ci, slot, status_id) = s.key {
                    if ci == char_idx {
                        return Some((slot, status_id, &s.state));
                    }
                }
                None
            })
            .collect()
    }

    pub fn character_statuses_vec(&self, char_idx: u8) -> Vec<&StatusEntry> {
        let f = StatusKeyFilter::Character(char_idx);
        self._status_entries.iter().filter(|&s| f.matches(s.key)).collect()
    }

    pub fn summon_statuses_vec(&self) -> Vec<&StatusEntry> {
        let f = StatusKeyFilter::Summon;
        self._status_entries.iter().filter(|&s| f.matches(s.key)).collect()
    }

    pub fn support_statuses_vec(&self) -> Vec<&StatusEntry> {
        self._status_entries
            .iter()
            .filter(|&s| matches!(s.key, StatusKey::Support(..)))
            .collect()
    }

    pub(crate) fn refresh_responds_to(&mut self) {
        let mut m1 = enum_set![];
        let mut t1 = enum_set![];
        let mut e1 = Default::default();
        for e in &self._status_entries {
            let si: StaticStatusImpl = match e.key {
                StatusKey::Team(status_id)
                | StatusKey::Character(_, status_id)
                | StatusKey::Equipment(_, _, status_id) => status_id.into(),
                StatusKey::Summon(summon_id) => summon_id.into(),
                StatusKey::Support(_, support_id) => support_id.into(),
            };
            m1 |= si.responds_to();
            t1 |= si.responds_to_triggers();
            e1 |= si.responds_to_events();
        }

        self.responds_to = m1;
        self.responds_to_triggers = t1;
        self.responds_to_events = e1;
    }

    pub(crate) fn apply_or_refresh_status(
        &mut self,
        path: StatusKey,
        status_spec: &'static Status,
        modifiers: &Option<StatusSpecModifier>,
    ) {
        if let Some(eff_state) = self.get_mut(path) {
            eff_state.refresh(status_spec);
            if let Some(modifiers) = modifiers {
                modifiers.modify(path, eff_state);
            }
        } else {
            let mut eff_state = AppliedEffectState::new(status_spec);
            if let Some(modifiers) = modifiers {
                modifiers.modify(path, &mut eff_state);
            }
            self.push_status_entry(StatusEntry::new(path, eff_state));
            let si: StaticStatusImpl = match path {
                StatusKey::Team(status_id) => status_id.into(),
                StatusKey::Summon(summon_id) => summon_id.into(),
                StatusKey::Character(_, status_id) => status_id.into(),
                StatusKey::Equipment(_, _, status_id) => status_id.into(),
                StatusKey::Support(_, support_id) => support_id.into(),
            };
            self.responds_to |= si.responds_to();
            self.responds_to_triggers |= si.responds_to_triggers();
            self.responds_to_events |= si.responds_to_events();
        }
    }

    pub(crate) fn clear_character_statuses<A: smallvec::Array<Item = (StatusId, AppliedEffectState)>>(
        &mut self,
        char_idx: u8,
        shifts_to_next_active: &mut SmallVec<A>,
    ) {
        self._status_entries.retain(|e| {
            let StatusKey::Character(status_char_idx, status_id) = e.key else { return true };
            if status_char_idx != char_idx {
                return true;
            }
            if status_id.get_status().shifts_to_next_active_on_death {
                shifts_to_next_active.push((status_id, e.state));
            }
            false
        });
    }

    pub fn has_shield_points(&self) -> bool {
        self._status_entries.iter().any(|e| match e.key {
            StatusKey::Team(status_id) | StatusKey::Character(_, status_id) | StatusKey::Equipment(_, _, status_id) => {
                status_id.get_status().usages_as_shield_points && e.state.get_usages() > 0
            }
            StatusKey::Summon(_) | StatusKey::Support(..) => false,
        })
    }

    /// Checks if there is any "cannot perform actions".
    /// NOTE: Only checks character statuses.
    pub fn cannot_perform_actions(&self, char_idx: u8) -> bool {
        self._status_entries.iter().any(|e| {
            if let StatusKey::Character(j, status_id) = e.key {
                if j == char_idx {
                    let si: StaticStatusImpl = status_id.into();
                    return si.responds_to().contains(RespondsTo::CannotPerformActions);
                }
            }
            false
        })
    }

    pub fn find_preparing_skill(&self, active_char_idx: u8) -> Option<(StatusKey, SkillId)> {
        if !self.responds_to(RespondsTo::PreparingSkill) {
            return None;
        }

        for e in &self._status_entries {
            let StatusKey::Character(_, status_id) = e.key else { continue };
            let si: StaticStatusImpl = status_id.into();
            if !si.responds_to().contains(RespondsTo::PreparingSkill) {
                continue;
            }
            if let Some(t) = si.preparing_skill(&e.state) {
                let char_idx = e.key.char_idx().expect("Prepare skills must be character statuses.");
                if char_idx != active_char_idx {
                    continue;
                }
                return Some((e.key, t));
            }
        }
        None
    }

    pub fn find_preparing_skill_with_status_key_and_turns_remaining(&self) -> Option<(SkillId, StatusKey, u8)> {
        if !self.responds_to(RespondsTo::PreparingSkill) {
            return None;
        }

        for e in &self._status_entries {
            let StatusKey::Character(_, status_id) = e.key else { continue };
            let si: StaticStatusImpl = status_id.into();
            if !si.responds_to().contains(RespondsTo::PreparingSkill) {
                continue;
            }
            if let Some(t) = si.preparing_skill(&e.state) {
                return Some((t, e.key, e.state.get_counter()));
            }
        }
        None
    }

    pub fn find_preparing_skill_status_entry_mut(&mut self) -> Option<(SkillId, &mut StatusEntry)> {
        if !self.responds_to(RespondsTo::PreparingSkill) {
            return None;
        }

        for e in &mut self._status_entries {
            let StatusKey::Character(_, status_id) = e.key else { continue };
            let si: StaticStatusImpl = status_id.into();
            if !si.responds_to().contains(RespondsTo::PreparingSkill) {
                continue;
            }
            if let Some(t) = si.preparing_skill(&e.state) {
                return Some((t, e));
            }
        }
        None
    }

    pub(crate) fn for_each_char_status_mut_retain<
        F: FnMut(StatusId, &mut AppliedEffectState) -> bool,
        G: FnMut(SummonId, &mut AppliedEffectState) -> bool,
        H: FnMut(SupportId, &mut AppliedEffectState) -> bool,
    >(
        &mut self,
        char_idx_opt: Option<u8>,
        mut f: F,
        mut g: G,
        mut h: H,
    ) {
        let c1 = self.count();
        self._status_entries.retain(|e| {
            let v = &mut e.state;
            match e.key {
                StatusKey::Team(status_id) => f(status_id, v),
                character_or_equipment!(i, status_id) if Some(i) == char_idx_opt => f(status_id, v),
                character_or_equipment!(_, _) => true,
                StatusKey::Summon(summon_id) => g(summon_id, v),
                StatusKey::Support(_, support_id) => h(support_id, v),
            }
        });
        if self.count() != c1 {
            self.refresh_responds_to();
        }
    }

    pub(crate) fn consume_statuses_first<
        P: Fn(StaticStatusImpl) -> bool,
        F: FnMut(&AppliedEffectState, StatusKey, StaticStatusImpl) -> Option<AppliedEffectResult>,
    >(
        &mut self,
        s: CharacterIndexSelector,
        check: P,
        mut func: F,
    ) -> bool {
        let mut found = false;
        self.consume_statuses(s, check, |es, sk, si| {
            if found {
                return None;
            }
            let res = func(es, sk, si);
            if res.is_some() {
                found = true
            }
            res
        })
    }

    #[inline]
    pub(crate) fn consume_statuses<
        P: Fn(StaticStatusImpl) -> bool,
        F: FnMut(&mut AppliedEffectState, StatusKey, StaticStatusImpl) -> Option<AppliedEffectResult>,
    >(
        &mut self,
        s: CharacterIndexSelector,
        check: P,
        mut func: F,
    ) -> bool {
        macro_rules! closure_body {
            ($eff_state: expr, $sk: expr, $found: ident, $get_status_impl: expr) => {{
                let si: StaticStatusImpl = $get_status_impl;
                if check(si) {
                    let status_key: StatusKey = $sk;
                    let mut_eff_state: &mut _ = &mut $eff_state;
                    let res = func(mut_eff_state, status_key, si);
                    ($eff_state).apply_change_and_check_retain(&mut $found, res, status_key.get_status())
                } else {
                    true
                }
            }};
        }

        let mut found = false;
        let c1 = self.count();
        self._status_entries.retain(|e| match e.key {
            StatusKey::Team(status_id) => {
                closure_body!(e.state, e.key, found, status_id.into())
            }
            StatusKey::Character(i, status_id) if s.selects(i) => {
                closure_body!(e.state, e.key, found, status_id.into())
            }
            StatusKey::Character(..) => true,
            StatusKey::Equipment(i, _, status_id) if s.selects(i) => {
                let res = closure_body!(e.state, e.key, found, status_id.into());
                if !res {
                    panic!("StatusCollection::consume_statuses: Cannot consume equipment status.")
                }
                true
            }
            StatusKey::Equipment(..) => true,
            StatusKey::Summon(summon_id) => {
                closure_body!(e.state, e.key, found, summon_id.into())
            }
            StatusKey::Support(_, support_id) => {
                closure_body!(e.state, e.key, found, support_id.into())
            }
        });
        if self.count() != c1 {
            self.refresh_responds_to();
        }
        found
    }

    pub(crate) fn consume_statuses_immutable<
        P: Fn(StaticStatusImpl) -> bool,
        F: FnMut(&AppliedEffectState, StatusKey, StaticStatusImpl) -> Option<AppliedEffectResult>,
    >(
        &self,
        s: CharacterIndexSelector,
        check: P,
        mut func: F,
    ) -> bool {
        let mut found = false;
        macro_rules! closure_body {
            ($eff_state: expr, $sk: expr, $found: ident, $get_status_impl: expr) => {{
                let si: StaticStatusImpl = $get_status_impl;
                found |= if check(si) {
                    let sk = $sk;
                    let es: &AppliedEffectState = &$eff_state;
                    func(es, sk, si).is_some()
                } else {
                    false
                }
            }};
        }

        for e in &self._status_entries {
            match e.key {
                StatusKey::Team(status_id) => {
                    closure_body!(e.state, e.key, found, status_id.into())
                }
                character_or_equipment!(i, status_id) => {
                    if s.selects(i) {
                        closure_body!(e.state, e.key, found, status_id.into())
                    }
                }
                StatusKey::Summon(summon_id) => {
                    closure_body!(e.state, e.key, found, summon_id.into())
                }
                StatusKey::Support(_, support_id) => {
                    closure_body!(e.state, e.key, found, support_id.into())
                }
            }
        }

        found
    }

    #[allow(dead_code)]
    pub(crate) fn for_each_status_mut_retain<F: FnMut(&'static Status, &mut AppliedEffectState) -> bool>(
        &mut self,
        mut f: F,
    ) {
        let c1 = self.count();
        self._status_entries.retain(|e| match e.key {
            StatusKey::Team(status_id) | character_or_equipment!(_, status_id) => {
                f(status_id.get_status(), &mut e.state)
            }
            StatusKey::Summon(summon_id) => f(summon_id.get_status(), &mut e.state),
            StatusKey::Support(_, support_id) => f(support_id.get_status(), &mut e.state),
        });
        if self.count() != c1 {
            self.refresh_responds_to();
        }
    }

    pub fn get_next_available_support_slot(&self) -> Option<SupportSlot> {
        for slot in SupportSlot::VALUES {
            let found = self._status_entries.iter().any(|s| match s.key {
                StatusKey::Support(slot1, _) => slot1 == slot,
                _ => false,
            });
            if !found {
                return Some(slot);
            }
        }
        None
    }

    pub(crate) fn set_status(&mut self, key: StatusKey, state: AppliedEffectState) -> bool {
        if let Some(found) = self._status_entries.iter_mut().find(|e| e.key == key) {
            found.state = state;
            return true;
        }
        self.push_status_entry(StatusEntry::new(key, state));
        false
    }

    #[inline]
    fn push_status_entry(&mut self, status_entry: StatusEntry) {
        let key = status_entry.key.sort_key();
        let n = self._status_entries.len();
        if self._status_entries.is_empty() || self._status_entries[n - 1].key.sort_key() <= key {
            self._status_entries.push(status_entry);
            return;
        }

        let mut ins_index = n;
        for (i, status_entry) in self._status_entries.iter().enumerate() {
            if status_entry.key.sort_key() > key {
                ins_index = i;
                break;
            }
        }
        self._status_entries.insert(ins_index, status_entry);
    }
}
