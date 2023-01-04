use crate::tcg_model::enums::{Element, Reaction};
use crate::types::char_state::{AppliedEffectResult, CharState};
use crate::types::command::*;
use crate::types::deal_dmg::{DealDMG, DealDMGType};

impl<'a, 'b, 'c, 'd, 'v, T> TriggerEventContext<'a, 'b, 'c, 'd, 'v, T> {
    #[inline]
    pub fn cmd_deal_dmg(&mut self, dmg_type: DealDMGType, dmg: u8, piercing_dmg_to_standby: u8) {
        self.out_cmds.push((
            *self.ctx_for_dmg,
            Command::DealDMG(DealDMG::new(dmg_type, dmg, piercing_dmg_to_standby)),
        ));
    }

    // TODO refactor code to use this method when possible
    #[inline]
    pub fn add_cmd(&mut self, cmd: Command) {
        self.out_cmds.push((*self.ctx_for_dmg, cmd));
    }

    #[inline]
    pub fn active_char_idx(&self) -> u8 {
        self.c.src_player_state.active_char_index
    }

    #[inline]
    pub fn src_char_idx(&self) -> Option<u8> {
        self.c.ctx.src.char_idx()
    }

    pub fn find_chararacter_for<F: Fn(&CharState) -> bool>(&self, f: F) -> Option<(usize, &CharState)> {
        self.c
            .src_player_state
            .char_states
            .iter()
            .enumerate()
            .find(|(_, c)| f(c))
    }

    #[inline]
    pub fn consume_counter<F: FnOnce(&mut Self, u8)>(&mut self, f: F) -> Option<AppliedEffectResult> {
        let c = self.c.eff_state.get_counter();
        if c == 0 {
            None
        } else {
            f(self, c);
            Some(AppliedEffectResult::SetCounter(c - 1))
        }
    }
}

impl<'a, 'b, 'c, 'd, 'v> TriggerEventContext<'a, 'b, 'c, 'd, 'v, XEvent> {
    #[inline]
    pub fn get_outgoing_dmg_ensuring_own_player(&self) -> Option<XEventDMG> {
        let XEvent::DMG(d) = self.event_id else { return None };
        if d.src_player_id == self.ctx_for_dmg.src_player_id {
            Some(d)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_outgoing_dmg_ensuring_attached_character(&self) -> Option<XEventDMG> {
        let XEvent::DMG(d) = self.event_id else { return None };
        let char_idx = self.status_key.char_idx().expect("Must be character/equipment status");
        if d.src_player_id == self.ctx_for_dmg.src_player_id && self.ctx_for_dmg.src.char_idx() == Some(char_idx) {
            Some(d)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_incoming_dmg_ensuring_own_player(&self) -> Option<XEventDMG> {
        let XEvent::DMG(d) = self.event_id else { return None };
        if d.src_player_id == self.ctx_for_dmg.src_player_id.opposite() {
            Some(d)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_incoming_dmg_ensuring_attached_character(&self) -> Option<XEventDMG> {
        let XEvent::DMG(d) = self.event_id else { return None };
        let char_idx = self.status_key.char_idx().expect("Must be character/equipment status");
        if d.src_player_id == self.ctx_for_dmg.src_player_id.opposite() && char_idx == d.tgt_char_idx {
            Some(d)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_outgoing_dmg(&self) -> bool {
        let XEvent::DMG(d) = self.event_id else { return false };
        d.src_player_id == self.ctx_for_dmg.src_player_id
    }

    #[inline]
    pub fn is_received_dmg(&self) -> bool {
        let XEvent::DMG(d) = self.event_id else { return false };
        d.src_player_id != self.ctx_for_dmg.src_player_id
    }

    #[inline]
    pub fn is_received_dmg_into_attached_character(&self) -> bool {
        let XEvent::DMG(d) = self.event_id else { return false };
        if d.src_player_id == self.ctx_for_dmg.src_player_id {
            return false;
        };
        let char_idx = self.status_key.char_idx().expect("Must be character/equipment status");
        d.tgt_char_idx == char_idx
    }

    #[inline]
    pub fn get_dmg_event_reaction(&self) -> Option<(Reaction, Option<Element>)> {
        if let XEvent::DMG(d) = self.event_id {
            d.reaction
        } else {
            None
        }
    }

    #[inline]
    pub fn get_event_skill_ensuring_own_player(&self) -> Option<XEventSkill> {
        let XEvent::Skill(evt_skill) = self.event_id else { return None };
        let src_player_id = self.ctx_for_dmg.src_player_id;
        if evt_skill.src_player_id == src_player_id {
            Some(evt_skill)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_event_skill_ensuring_opponent_player(&self) -> Option<XEventSkill> {
        let XEvent::Skill(evt_skill) = self.event_id else { return None };
        let src_player_id = self.ctx_for_dmg.src_player_id;
        if evt_skill.src_player_id == src_player_id.opposite() {
            Some(evt_skill)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_event_skill_ensuring_attached_character(&self) -> Option<XEventSkill> {
        let XEvent::Skill(evt_skill) = self.event_id else { return None };
        let src_player_id = self.ctx_for_dmg.src_player_id;
        let char_idx = self.status_key.char_idx().expect("Must be character/equipment status");
        let src_char_idx = self.ctx_for_dmg.src.char_idx()?;
        if src_char_idx == char_idx && evt_skill.src_player_id == src_player_id {
            Some(evt_skill)
        } else {
            None
        }
    }

    #[inline]
    pub fn attached_character_is_active(&self) -> bool {
        let char_idx = self.status_key.char_idx().expect("Must be character/equipment status");
        char_idx == self.active_char_idx()
    }
}
