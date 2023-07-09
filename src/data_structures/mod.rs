pub mod capped_list;

pub mod linked_list;

pub use linked_list::*;

pub type List8<T> = capped_list::CappedLengthList8<T>;

pub type CommandList<T> = smallvec::SmallVec<[T; 8]>;

pub type StatusEntryList<T> = smallvec::SmallVec<[T; 8]>;

pub type ActionList<T> = smallvec::SmallVec<[T; 16]>;

#[macro_export]
macro_rules! list8 {
    [] => ($crate::data_structures::List8::L08);
    [$v0: expr $(,)?] => ($crate::data_structures::List8::L18($v0));
    [$v0: expr, $v1: expr $(,)?] => ($crate::data_structures::List8::L28($v0, $v1));
    [$v0: expr, $v1: expr, $v2: expr $(,)?] => ($crate::data_structures::List8::L38($v0, $v1, $v2));
    [$v0: expr, $v1: expr, $v2: expr, $v3: expr $(,)?] => ($crate::data_structures::List8::L48($v0, $v1, $v2, $v3));
}

#[macro_export]
macro_rules! cmd_list {
    () => {
        $crate::smallvec::SmallVec::<[_; 8]>::new()
    };
    ($($e : expr),+ $(,)?) => {
        $crate::smallvec::smallvec![$($e),+]
    }
}

#[macro_export]
macro_rules! action_list {
    () => {
        $crate::smallvec::SmallVec::<[_; 16]>::new()
    };
    ($($e : expr),+ $(,)?) => {
        $crate::smallvec::smallvec![$($e),+]
    }
}

pub type Vector<T> = smallvec::SmallVec<[T; 4]>;

#[macro_export]
macro_rules! vector {
    () => {
        $crate::smallvec::SmallVec::<[_; 4]>::new()
    };
    ($($e : expr),+ $(,)?) => {
        $crate::smallvec::smallvec![$($e),+]
    }
}
