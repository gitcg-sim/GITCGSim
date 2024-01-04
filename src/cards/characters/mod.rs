// Generated code
use crate::types::{card_defs::*, command::*, game_state::*};
use crate::{decl_status_impl_type, decl_summon_impl_type, list8};

use crate::cards::{builders::*, ids::*};
use crate::data_structures::CommandList;
use crate::status_impls::prelude::*;
use crate::status_impls::primitives::all::*;

use super::ids::lookup::GetCharCard;

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
    crate::__generated_char_reexports!();
}

impl GetCharCard for CharId {
    #[inline]
    fn get_char_card(self: CharId) -> &'static CharCard {
        crate::__generated_enum_cases!(CharId, self, &C)
    }
}
