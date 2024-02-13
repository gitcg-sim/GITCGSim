use crate::types::card_defs::Skill;

use super::ids::*;

impl GetSkill for SkillId {
    fn get_skill(self) -> &'static Skill {
        crate::cards::characters::find_skill_precomputed(self)
    }
}
