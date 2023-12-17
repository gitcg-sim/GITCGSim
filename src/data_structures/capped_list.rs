use super::const_default::*;

/// A variable-length list with at most 8 elements that can be constructed
/// in the const context.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub struct CappedLengthList8<T: ConstDefault> {
    pub len: u8,
    pub array: [T; 8],
}

impl<T: ConstDefault> CappedLengthList8<T> {
    pub const EMPTY: Self = Self {
        len: 0,
        array: <[T; 8] as ConstDefault>::DEFAULT,
    };

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn len(&self) -> u8 {
        self.len
    }

    pub fn slice(&self) -> &[T] {
        &self.array[0..(self.len as usize)]
    }
}

impl<T: ConstDefault + Copy> CappedLengthList8<T> {
    pub fn fold_copy<A, F: FnMut(A, T) -> A>(&self, init: A, f: F) -> A {
        self.slice().iter().copied().fold(init, f)
    }

    pub fn to_vec_copy(&self) -> smallvec::SmallVec<[T; 8]> {
        self.slice().into()
    }

    pub const fn from_slice_copy(slice: &[T]) -> Self {
        let len = slice.len();
        let len = if len > 8 { 8 } else { len as u8 };
        // mut variables are not supported in const context
        let array = match len {
            0 => [
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
            ],
            1 => [
                slice[0],
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
            ],
            2 => [
                slice[0],
                slice[1],
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
            ],
            3 => [
                slice[0],
                slice[1],
                slice[2],
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
            ],
            4 => [
                slice[0],
                slice[1],
                slice[2],
                slice[3],
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
            ],
            5 => [
                slice[0],
                slice[1],
                slice[2],
                slice[3],
                slice[4],
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
            ],
            6 => [
                slice[0],
                slice[1],
                slice[2],
                slice[3],
                slice[4],
                slice[5],
                ConstDefault::DEFAULT,
                ConstDefault::DEFAULT,
            ],
            7 => [
                slice[0],
                slice[1],
                slice[2],
                slice[3],
                slice[4],
                slice[5],
                slice[6],
                ConstDefault::DEFAULT,
            ],
            _ => [
                slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
            ],
        };
        Self { len, array }
    }
}

impl<T: ConstDefault + enumset::EnumSetTypeWithRepr> CappedLengthList8<T> {
    pub fn to_enum_set(self) -> enumset::EnumSet<T> {
        self.fold_copy(Default::default(), |x, y| x | y)
    }
}

// Trait impls
impl<T: ConstDefault> ConstDefault for CappedLengthList8<T> {
    const DEFAULT: Self = Self::EMPTY;
}

impl<T: ConstDefault> Default for CappedLengthList8<T> {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl<T: ConstDefault + Copy, A: smallvec::Array<Item = T>> From<smallvec::SmallVec<A>> for CappedLengthList8<T> {
    fn from(v: smallvec::SmallVec<A>) -> Self {
        Self::from_slice_copy(&v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SLICE: [u8; 8] = [222, 54, 2, 14, 52, 120, 34, 224];

    #[test]
    fn from_slice_values_are_preserved() {
        for len in 0..=8usize {
            let list8 = CappedLengthList8::from_slice_copy(&SLICE[0..len]);
            let slice = list8.slice();
            assert_eq!(len, list8.len() as usize);
            assert_eq!(&SLICE[0..len], slice);
        }
    }

    #[test]
    fn fold() {
        for len in 0..=8usize {
            let list8 = CappedLengthList8::from_slice_copy(&SLICE[0..len]);
            let fold1 = list8.fold_copy(String::default(), |a, b| format!("{a}, {b}"));
            let fold2 = SLICE[0..len].iter().fold(String::default(), |a, b| format!("{a}, {b}"));
            assert_eq!(fold1, fold2);
        }
    }
}
