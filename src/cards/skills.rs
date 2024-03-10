use crate::types::card_defs::Skill;

use crate::cards::{characters::*, ids::*};

macro_rules! for_each_enum {
    ($var: ident : $type: ty => $blk: block) => {{
        let n: usize = <$type as enum_map::Enum>::LENGTH;
        let mut i = 0usize;
        while i < n {
            let u8_value = i as u8;
            let $var: $type = unsafe { std::mem::transmute(u8_value) };
            $blk;
            i += 1;
        }
    }};
}

const fn skill_id_equals(a: SkillId, b: SkillId) -> bool {
    (a as u8) == (b as u8)
}

const fn find_skill_by_char_id(char_id: CharId, skill_id: SkillId) -> Option<&'static Skill> {
    let skills = char_id.__generated_lookup_skills();
    let n = skills.len();
    let mut i = 0;
    while i < n {
        let (skill_id1, skill) = &skills[i];
        if skill_id_equals(*skill_id1, skill_id) {
            return Some(skill);
        }
        i += 1;
    }
    None
}

const fn find_skill(skill_id: SkillId) -> Option<&'static Skill> {
    for_each_enum!(char_id: CharId => {
        if let Some(res) = find_skill_by_char_id(char_id, skill_id) {
            return Some(res);
        }
    });
    None
}

const fn precomputed_find_skill() -> [&'static Skill; <SkillId as enum_map::Enum>::LENGTH] {
    let mut res = [&yoimiya::NIWABI_FIRE_DANCE; <SkillId as enum_map::Enum>::LENGTH];
    for_each_enum!(skill_id: SkillId => {
        let idx = skill_id as u8 as usize;
        if let Some(skill) = find_skill(skill_id) {
            res[idx] = skill;
        } else {
            panic!("failed to find the `Skill` corresponding to a particular enum case of SkillId");
        }
    });
    res
}

const PRECOMPUTED_FIND_SKILL: [&Skill; <SkillId as enum_map::Enum>::LENGTH] = precomputed_find_skill();

const fn find_skill_precomputed(skill_id: SkillId) -> &'static Skill {
    PRECOMPUTED_FIND_SKILL[skill_id as u8 as usize]
}

impl GetSkill for SkillId {
    fn skill(self) -> &'static Skill {
        find_skill_precomputed(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_skill_by_char_id_returns_correct_result() {
        assert_eq!(
            None,
            find_skill_by_char_id(CharId::Yoimiya, SkillId::Akara).map(|s| s.name)
        );
        assert_eq!(
            Some(yoimiya::NIWABI_FIRE_DANCE.name),
            find_skill_by_char_id(CharId::Yoimiya, SkillId::NiwabiFireDance).map(|s| s.name)
        );
        assert_eq!(
            Some(nahida::AKARA.name),
            find_skill_by_char_id(CharId::Nahida, SkillId::Akara).map(|s| s.name)
        );
    }

    #[test]
    fn find_skill_is_same_as_get_skill_exhaustive() {
        for_each_enum!(skill_id: SkillId => {
            assert_eq!(Some(skill_id.skill().name), find_skill(skill_id).map(|s| s.name));
        });
    }

    #[test]
    fn find_skill_precomputed_is_same_as_find_skill_exhaustive() {
        for_each_enum!(skill_id: SkillId => {
            assert_eq!(find_skill(skill_id).expect("find_skill").name, find_skill_precomputed(skill_id).name);
        });
    }

    #[test]
    fn find_skill_names_are_distinct_exhaustive() {
        let mut set = std::collections::BTreeSet::new();
        let mut v = vec![];
        for_each_enum!(skill_id: SkillId => {
            let name = find_skill(skill_id).map(|s| s.name).expect("find_skill must not return None");
            set.insert(name);
            v.push(name);
        });
        v.sort();
        assert_eq!(<SkillId as enum_map::Enum>::LENGTH, set.len());
    }

    macro_rules! test_for_each_enum {
        ($type: ty) => {
            let mut values: Vec<String> = vec![];
            for i in 0..<$type as enum_map::Enum>::LENGTH {
                let enum_value = <$type as enum_map::Enum>::from_usize(i);
                values.push(format!("{enum_value:?}"));
            }
            values.sort();
            let mut values_const: Vec<String> = vec![];
            for_each_enum!(id: $type => {
                values_const.push(format!("{id:?}"));
            });
            values_const.sort();
            assert_eq!(values, values_const);
        }
    }

    #[test]
    fn for_each_enum_char_id() {
        test_for_each_enum!(CharId);
    }

    #[test]
    fn for_each_enum_skill_id() {
        test_for_each_enum!(SkillId);
    }

    #[test]
    fn for_each_enum_status_id() {
        test_for_each_enum!(StatusId);
    }

    #[test]
    fn for_each_enum_summon_id() {
        test_for_each_enum!(SummonId);
    }

    #[test]
    fn for_each_enum_card_id() {
        test_for_each_enum!(CardId);
    }
}
