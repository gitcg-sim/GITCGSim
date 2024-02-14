mod generated_code;

#[macro_export]
#[doc(hidden)]
macro_rules! __mapping {
    ($expr: expr $(,)?) => {
        $expr
    };
    ($expr: expr , | $val: ident | $blk: block $(,)?) => {{
        let $val = $expr;
        $blk
    }};
}

mod traits {
    use crate::types::{
        card_defs::{Card, CharCard, Skill, Status},
        card_impl::CardImpl,
        status_impl::StatusImpl,
    };

    pub trait GetCharCard
    where
        Self: Sized,
    {
        fn get_char_card(self) -> &'static CharCard;
    }

    pub trait GetSkill
    where
        Self: Sized,
    {
        fn get_skill(self) -> &'static Skill;
    }

    pub trait GetCard
    where
        Self: Sized,
    {
        fn get_card(self) -> &'static Card;

        #[inline]
        fn get_card_impl(self) -> Option<&'static dyn CardImpl> {
            self.get_card().card_impl
        }
    }

    pub trait GetStatus
    where
        Self: Sized,
    {
        fn get_status(self) -> &'static Status;

        fn get_status_impl(self) -> &'static dyn StatusImpl;
    }
}

pub use traits::*;
