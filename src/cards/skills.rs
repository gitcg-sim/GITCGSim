use crate::types::card_defs::Skill;

use super::ids::*;

impl GetSkill for SkillId {
    fn get_skill(self) -> &'static Skill {
        use crate::cards::characters::char_reexports::*;
        crate::__generated_skills_cases!(self)
    }
}
