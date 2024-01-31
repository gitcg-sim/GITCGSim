pub mod const_default;

pub use const_default::*;

pub mod capped_list;

pub type List8<T> = capped_list::CappedLengthList8<T>;

pub type CommandList<T> = smallvec::SmallVec<[T; 8]>;

pub type StatusEntryList<T> = smallvec::SmallVec<[T; 8]>;

pub type ActionList<T> = smallvec::SmallVec<[T; 16]>;

/// Creates a [List8] containing the arguments.
///
/// Can be used in `const` contexts.
#[macro_export]
macro_rules! list8 {
    [] => ($crate::data_structures::List8::EMPTY);
    [$($v: expr),+ $(,)?] => ($crate::data_structures::List8::from_slice_copy(&[$($v),+]));
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
