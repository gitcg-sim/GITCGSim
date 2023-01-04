use super::*;
use crate::cards::summons::burning_flame;
use crate::types::{card_defs::Status, status_impl::StatusImpl};

#[allow(unused_imports)]
impl GetStatus for SummonId {
    #[inline]
    fn get_status(self) -> &'static Status {
        use crate::cards::characters::char_reexports::*;
        crate::__generated_enum_cases!(SummonId, self, &S)
    }

    #[inline]
    fn get_status_impl(self) -> &'static dyn StatusImpl {
        use crate::cards::characters::char_reexports::*;
        crate::__generated_enum_cases!(SummonId, self, &I)
    }
}
