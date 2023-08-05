use crate::cards::ids::*;
use crate::dispatcher_ops::state_ops::check_valid_char_index;
use crate::status_impls::prelude::WeaponType;
use crate::tcg_model::enums::SkillType;
use crate::types::{card_defs::*, command::*, game_state::*};

impl<'a, 'b, 'c, 'v, D> StatusImplContext<'a, 'b, 'c, 'v, D> {
    #[inline]
    pub fn src_char_id(&self) -> Option<CharId> {
        let ci = self.ctx.src.char_idx()?;
        let cs = &self.src_player_state.char_states;
        if check_valid_char_index(cs, ci) {
            return Some(cs[ci as usize].char_id);
        }
        None
    }

    #[inline]
    pub fn src_char_card(&self) -> Option<&'static CharCard> {
        self.src_char_id().map(CharId::get_char_card)
    }

    #[inline]
    pub fn skill_id(&self) -> Option<SkillId> {
        self.ctx.src.skill_id()
    }

    #[inline]
    pub fn skill(&self) -> Option<&'static Skill> {
        self.skill_id().map(SkillId::get_skill)
    }

    #[inline]
    pub fn skill_type(&self) -> Option<SkillType> {
        self.skill().map(|s| s.skill_type)
    }

    #[inline]
    pub fn weapon_type(&self) -> Option<WeaponType> {
        self.get_src_character_state().map(|c| c.char_id.get_char_card().weapon)
    }

    #[inline]
    pub fn is_casted_by_character(&self, char_id: CharId) -> bool {
        if let Some(i) = self.ctx.src.char_idx() {
            let cs = &self.src_player_state.char_states;
            check_valid_char_index(cs, i) && char_id == cs[i as usize].char_id
        } else {
            false
        }
    }

    #[inline]
    pub fn get_src_character_state(&self) -> Option<&CharState> {
        self.ctx.src.char_idx().and_then(|ci| self.get_character_state(ci))
    }

    #[inline]
    pub fn get_character_state(&self, char_idx: u8) -> Option<&CharState> {
        let cs = self.src_player_state.char_states;
        if check_valid_char_index(cs, char_idx) {
            return Some(&cs[char_idx as usize]);
        }
        None
    }

    #[inline]
    pub fn is_switched_into_character(&self, char_id: CharId) -> bool {
        if let Some(tgt_char_idx) = self.ctx.src.dst_char_idx() {
            if let Some(cs) = self.get_character_state(tgt_char_idx) {
                return cs.char_id == char_id;
            }
        }
        false
    }

    #[inline]
    pub fn is_switch(&self) -> bool {
        matches!(self.ctx.src, CommandSource::Switch { .. })
    }

    #[inline]
    /// Skill is Normal Attack and dice counter pre-cast is even
    pub fn is_charged_attack(&self) -> bool {
        self.skill_type() == Some(SkillType::NormalAttack)
            && self.src_player_state.flags.contains(PlayerFlag::ChargedAttack)
    }

    #[inline]
    pub fn has_talent_equipped(&self) -> bool {
        let check_for_status = |status: &'static Status| -> bool {
            if status.applies_to_opposing {
                unimplemented!()
            }
            let char_id = status.casted_by_character();
            self.src_player_state
                .char_states
                .iter()
                .any(|cs| cs.char_id == char_id && cs.has_talent_equipped())
        };

        match self.status_key {
            StatusKey::Character(..) => self
                .get_src_character_state()
                .map(CharState::has_talent_equipped)
                .unwrap_or(false),
            StatusKey::Team(status_id) => check_for_status(status_id.get_status()),
            StatusKey::Summon(summon_id) => check_for_status(summon_id.get_status()),
            StatusKey::Equipment(..) => false,
            StatusKey::Support(..) => false,
        }
    }
}
