use enumset::EnumSetTypeWithRepr;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, Ord, Default)]
pub enum CappedLengthList8<T> {
    #[default]
    L08,
    L18(T),
    L28(T, T),
    L38(T, T, T),
    L48(T, T, T, T),
    L58(T, T, T, T, T),
    L68(T, T, T, T, T, T),
    L78(T, T, T, T, T, T, T),
    L88(T, T, T, T, T, T, T, T),
}

impl<T: Copy> CappedLengthList8<T> {
    pub fn to_vec(self) -> smallvec::SmallVec<[T; 8]> {
        match self {
            CappedLengthList8::L08 => smallvec::smallvec![],
            CappedLengthList8::L18(v0) => smallvec::smallvec![v0],
            CappedLengthList8::L28(v0, v1) => smallvec::smallvec![v0, v1],
            CappedLengthList8::L38(v0, v1, v2) => smallvec::smallvec![v0, v1, v2],
            CappedLengthList8::L48(v0, v1, v2, v3) => smallvec::smallvec![v0, v1, v2, v3],
            CappedLengthList8::L58(v0, v1, v2, v3, v4) => smallvec::smallvec![v0, v1, v2, v3, v4],
            CappedLengthList8::L68(v0, v1, v2, v3, v4, v5) => smallvec::smallvec![v0, v1, v2, v3, v4, v5],
            CappedLengthList8::L78(v0, v1, v2, v3, v4, v5, v6) => smallvec::smallvec![v0, v1, v2, v3, v4, v5, v6],
            CappedLengthList8::L88(v0, v1, v2, v3, v4, v5, v6, v7) => {
                smallvec::smallvec![v0, v1, v2, v3, v4, v5, v6, v7]
            }
        }
    }
}

impl<T: Copy + EnumSetTypeWithRepr> CappedLengthList8<T> {
    pub fn to_enum_set(self) -> enumset::EnumSet<T> {
        self.fold(Default::default(), |x, &y| x | y)
    }
}

impl<T: Copy, A: smallvec::Array<Item = T>> From<smallvec::SmallVec<A>> for CappedLengthList8<T> {
    fn from(v: smallvec::SmallVec<A>) -> Self {
        match v.len() {
            0 => Self::L08,
            1 => Self::L18(v[0]),
            2 => Self::L28(v[0], v[1]),
            3 => Self::L38(v[0], v[1], v[2]),
            4 => Self::L48(v[0], v[1], v[2], v[3]),
            5 => Self::L58(v[0], v[1], v[2], v[3], v[4]),
            6 => Self::L68(v[0], v[1], v[2], v[3], v[4], v[5]),
            7 => Self::L78(v[0], v[1], v[2], v[3], v[4], v[5], v[6]),
            _ => Self::L88(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]),
        }
    }
}

impl<T> CappedLengthList8<T> {
    pub fn is_empty(&self) -> bool {
        matches!(self, CappedLengthList8::L08)
    }

    pub fn len(&self) -> u8 {
        match self {
            CappedLengthList8::L08 => 0,
            CappedLengthList8::L18(_) => 1,
            CappedLengthList8::L28(_, _) => 2,
            CappedLengthList8::L38(_, _, _) => 3,
            CappedLengthList8::L48(_, _, _, _) => 4,
            CappedLengthList8::L58(_, _, _, _, _) => 5,
            CappedLengthList8::L68(_, _, _, _, _, _) => 6,
            CappedLengthList8::L78(_, _, _, _, _, _, _) => 7,
            CappedLengthList8::L88(_, _, _, _, _, _, _, _) => 8,
        }
    }

    pub fn fold<A>(&self, init: A, f: fn(A, &T) -> A) -> A {
        match self {
            CappedLengthList8::L08 => init,
            CappedLengthList8::L18(v0) => f(init, v0),
            CappedLengthList8::L28(v0, v1) => f(f(init, v0), v1),
            CappedLengthList8::L38(v0, v1, v2) => f(f(f(init, v0), v1), v2),
            CappedLengthList8::L48(v0, v1, v2, v3) => f(f(f(f(init, v0), v1), v2), v3),
            CappedLengthList8::L58(v0, v1, v2, v3, v4) => f(f(f(f(f(init, v0), v1), v2), v3), v4),
            CappedLengthList8::L68(v0, v1, v2, v3, v4, v5) => f(f(f(f(f(f(init, v0), v1), v2), v3), v4), v5),
            CappedLengthList8::L78(v0, v1, v2, v3, v4, v5, v6) => f(f(f(f(f(f(f(init, v0), v1), v2), v3), v4), v5), v6),
            CappedLengthList8::L88(v0, v1, v2, v3, v4, v5, v6, v7) => {
                f(f(f(f(f(f(f(f(init, v0), v1), v2), v3), v4), v5), v6), v7)
            }
        }
    }
}
