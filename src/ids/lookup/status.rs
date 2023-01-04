use super::*;

use crate::cards::equipment::{artifact::*, talent::*, weapon::*};
use crate::cards::{event::*, statuses::*};
use crate::types::{card_defs::Status, status_impl::StatusImpl};

#[allow(unused_imports)]
impl GetStatus for StatusId {
    #[inline]
    fn get_status(self) -> &'static Status {
        use crate::cards::characters::char_reexports::*;
        crate::__generated_enum_cases!(StatusId, self, &S)
    }

    #[inline]
    fn get_status_impl(self) -> &'static dyn StatusImpl {
        use crate::cards::characters::char_reexports::*;
        crate::__generated_enum_cases!(StatusId, self, &I)
    }
}
