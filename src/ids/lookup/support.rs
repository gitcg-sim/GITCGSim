use super::*;

use crate::types::{card_defs::Status, status_impl::StatusImpl};
use crate::{cards::support::*, ids::SupportId};

#[allow(unused_imports)]
impl GetStatus for SupportId {
    #[inline]
    fn get_status(self) -> &'static Status {
        use crate::cards::characters::char_reexports::*;
        crate::__generated_enum_cases!(SupportId, self, &S)
    }

    #[inline]
    fn get_status_impl(self) -> &'static dyn StatusImpl {
        use crate::cards::characters::char_reexports::*;
        crate::__generated_enum_cases!(SupportId, self, &I)
    }
}
