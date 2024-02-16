use crate::types::{card_defs::*, command::*, game_state::*};
use crate::{decl_status_impl_type, decl_summon_impl_type, list8};

use crate::cards::{builders::*, ids::*};
use crate::data_structures::CommandList;
use crate::status_impls::prelude::*;
use crate::status_impls::primitives::all::*;

use super::ids::GetCharCard;

pub mod albedo;
pub mod amber;
pub mod arataki_itto;
pub mod barbara;
pub mod beidou;
pub mod bennett;
pub mod candace;
pub mod chongyun;
pub mod collei;
pub mod cyno;
pub mod diluc;
pub mod diona;
pub mod eula;
pub mod fatui_pyro_agent;
pub mod fischl;
pub mod ganyu;
pub mod hu_tao;
pub mod jadeplume_terrorshroom;
pub mod jean;
pub mod kaeya;
pub mod kamisato_ayaka;
pub mod kamisato_ayato;
pub mod keqing;
pub mod klee;
pub mod kujou_sara;
pub mod mona;
pub mod nahida;
pub mod nilou;
pub mod ningguang;
pub mod noelle;
pub mod qiqi;
pub mod raiden_shogun;
pub mod razor;
pub mod rhodeia_of_loch;
pub mod sangonomiya_kokomi;
pub mod shenhe;
pub mod stonehide_lawachurl;
pub mod sucrose;
pub mod tartaglia;
pub mod tighnari;
pub mod venti;
pub mod wanderer;
pub mod xiangling;
pub mod xiao;
pub mod xingqiu;
pub mod yae_miko;
pub mod yanfei;
pub mod yaoyao;
pub mod yoimiya;
pub mod zhongli;

pub(crate) mod char_reexports {
    pub use crate::ids::__generated_char_reexports::*;
}

impl GetCharCard for CharId {
    #[inline]
    fn get_char_card(self: CharId) -> &'static CharCard {
        self.__generated_lookup_char_card()
    }
}

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

const fn get_precomputed_find_skill() -> [&'static Skill; <SkillId as enum_map::Enum>::LENGTH] {
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

const PRECOMPUTED_FIND_SKILL: [&Skill; <SkillId as enum_map::Enum>::LENGTH] = get_precomputed_find_skill();

pub(crate) const fn find_skill_precomputed(skill_id: SkillId) -> &'static Skill {
    PRECOMPUTED_FIND_SKILL[skill_id as u8 as usize]
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
            assert_eq!(Some(skill_id.get_skill().name), find_skill(skill_id).map(|s| s.name));
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
