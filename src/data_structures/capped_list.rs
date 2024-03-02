use core::marker::PhantomData;

use super::const_default::*;

/// A variable-length list with at most N items indexed and with length of [u8].
///
/// Unlike [Vec] or [smallvec::SmallVec], [CappedLengthList8] can
/// be constructed with specific elements in `const` contexts
/// through the [crate::list8] macro.
/// Like slices, [CappedLengthList8] can be copied if `T : Copy`.
///
/// To avoid dealing with uninitialized values (unsafe code),
/// the element type `T` must be [ConstDefault].
///
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "[T; N]: serde::Serialize",
        deserialize = "[T; N]: serde::Deserialize<'de>"
    ))
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub struct CappedLengthList8<T: ConstDefault, const N: usize> {
    pub(crate) len: u8,
    pub(crate) array: [T; N],
    _marker: PhantomData<()>,
}

// TODO rename
impl<T: ConstDefault, const N: usize> CappedLengthList8<T, N> {
    pub const LENGTH_RESTRICTION_32: PhantomData<()> = {
        if N > 32 {
            panic!("N must not be greater than 32.");
        }
        PhantomData
    };

    pub const EMPTY: Self = Self {
        len: 0,
        array: <[T; N] as ConstDefault>::DEFAULT,
        _marker: Self::LENGTH_RESTRICTION_32,
    };

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub const fn len(&self) -> u8 {
        self.len
    }

    #[inline(always)]
    pub fn slice(&self) -> &[T] {
        &self.array[0..(self.len as usize)]
    }

    #[inline(always)]
    pub fn slice_mut(&mut self) -> &mut [T] {
        &mut self.array[0..(self.len as usize)]
    }
}

impl<T: ConstDefault, const N: usize> crate::std_subset::ops::Index<u8> for CappedLengthList8<T, N> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: u8) -> &T {
        &self.slice()[index as usize]
    }
}

impl<T: ConstDefault, const N: usize> crate::std_subset::ops::IndexMut<u8> for CappedLengthList8<T, N> {
    #[inline(always)]
    fn index_mut(&mut self, index: u8) -> &mut T {
        &mut self.slice_mut()[index as usize]
    }
}

macro_rules! from_fixed_slice_impl {
    ($($n: literal),+ ; $N: literal) => {
        $(
            impl<T: ConstDefault + Copy> From<[T; $n]> for CappedLengthList8<T, $N> {
                #[inline]
                fn from(value: [T; $n]) -> Self {
                    Self::from_fixed_size_slice_copy::<$n>(&value)
                }
            }
        )+
    }
}
from_fixed_slice_impl!(0, 1, 2, 3, 4, 5, 6, 7, 8; 8);

impl<T: ConstDefault + Copy, const N: usize> CappedLengthList8<T, N> {
    pub fn fold_copy<A, F: FnMut(A, T) -> A>(&self, init: A, f: F) -> A {
        self.slice().iter().copied().fold(init, f)
    }

    pub fn to_vec_copy(&self) -> smallvec::SmallVec<[T; 8]> {
        self.slice().into()
    }

    /// Panics: When `N > M`
    #[inline]
    pub(crate) fn from_fixed_size_slice_copy<const M: usize>(slice: &[T; M]) -> Self {
        if M > N {
            panic!("Source slice length ({M}) is too large (> {N})");
        }
        let mut array = [ConstDefault::DEFAULT; N];
        array[0..M].copy_from_slice(slice);
        Self {
            len: M as u8,
            array,
            _marker: Self::LENGTH_RESTRICTION_32,
        }
    }

    pub(crate) const fn from_slice_copy(slice: &[T]) -> Self {
        let len = slice.len();
        let len = if len > N { N as u8 } else { len as u8 };
        let mut array = [ConstDefault::DEFAULT; N];
        let mut i = 0;
        let len_usize = len as usize;
        while i < len_usize {
            array[i] = slice[i];
            i += 1;
        }
        Self {
            len,
            array,
            _marker: Self::LENGTH_RESTRICTION_32,
        }
    }
}

impl<T: ConstDefault + enumset::EnumSetTypeWithRepr> CappedLengthList8<T, 8> {
    pub fn to_enum_set(self) -> enumset::EnumSet<T> {
        self.fold_copy(Default::default(), |x, y| x | y)
    }
}

// Trait impls
impl<T: ConstDefault, const N: usize> ConstDefault for CappedLengthList8<T, N> {
    const DEFAULT: Self = Self::EMPTY;
}

impl<T: ConstDefault, const N: usize> Default for CappedLengthList8<T, N> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<T: ConstDefault + Copy, A: smallvec::Array<Item = T>, const N: usize> From<smallvec::SmallVec<A>>
    for CappedLengthList8<T, N>
{
    fn from(v: smallvec::SmallVec<A>) -> Self {
        Self::from_slice_copy(&v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SLICE: [u8; 8] = [222, 54, 2, 14, 52, 120, 34, 224];

    #[test]
    fn from() {
        const EMPTY: [u8; 0] = [];
        assert_eq!(EMPTY, CappedLengthList8::<u8, 8>::from([]).slice());
        assert_eq!([1], CappedLengthList8::from([1]).slice());
        assert_eq!([1, 2, 3], CappedLengthList8::from([1, 2, 3]).slice());
    }

    #[test]
    fn from_slice_values_are_preserved() {
        for len in 0..=8usize {
            let list8 = CappedLengthList8::<u8, 8>::from_slice_copy(&SLICE[0..len]);
            let slice = list8.slice();
            assert_eq!(len, list8.len() as usize);
            assert_eq!(&SLICE[0..len], slice);
        }
    }

    #[test]
    fn fold() {
        for len in 0..=8usize {
            let list8 = CappedLengthList8::<u8, 8>::from_slice_copy(&SLICE[0..len]);
            let fold1 = list8.fold_copy(String::default(), |a, b| format!("{a}, {b}"));
            let fold2 = SLICE[0..len].iter().fold(String::default(), |a, b| format!("{a}, {b}"));
            assert_eq!(fold1, fold2);
        }
    }
}
