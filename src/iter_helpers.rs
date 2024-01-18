use enumset::{EnumSet, EnumSetType};

/// Iterate through a slice, with the iterator owning it.
pub struct IterSliceCopied<V: Copy, const N: usize> {
    slice: [V; N],
    index: usize,
    len: usize,
}

/// Iterate through the provided iterator if exists.
pub struct IterOption<T: Iterator<Item = V>, V> {
    iter: Option<T>,
}

/// Iterate through an iterator, discarding already-seen items.
/// Already-seen items are kept track in an [enumset::EnumSet].
pub struct IterDistinct<T: Iterator<Item = V>, V: EnumSetType> {
    existing: EnumSet<V>,
    iter: T,
}

/// Iterate through one of 2 different iterators with the same `Item`.
pub enum IterSwitch<A: Iterator<Item = C>, B: Iterator<Item = C>, C> {
    Left(A),
    Right(B),
}

pub struct IterLazyChain<A: Iterator<Item = C>, B: Iterator<Item = C>, C, F: Fn(bool) -> B> {
    iter: IterSwitch<A, B, C>,
    found: bool,
    get_second: F,
}

pub struct IterLazyChain3<
    A: Iterator<Item = Item>,
    B: Iterator<Item = Item>,
    C: Iterator<Item = Item>,
    Item,
    Env: Copy,
    F: FnMut(Env) -> B,
    G: FnMut(Env) -> C,
> {
    iter: IterSwitch<A, IterSwitch<B, C, Item>, Item>,
    env: Env,
    get_second: F,
    get_third: G,
}

impl<V: Eq + Copy, const N: usize> IterSliceCopied<V, N> {
    pub fn new(slice: [V; N], len: usize) -> Self {
        Self { slice, index: 0, len }
    }
}

impl<T: Iterator<Item = V>, V: EnumSetType> IterDistinct<T, V> {
    pub fn new(iter: T) -> Self {
        Self {
            existing: Default::default(),
            iter,
        }
    }
}

impl<T: Iterator<Item = V>, V> IterOption<T, V> {
    pub fn new(iter: Option<T>) -> Self {
        Self { iter }
    }
}

impl<A: Iterator<Item = C>, B: Iterator<Item = C>, C, F: Fn(bool) -> B> IterLazyChain<A, B, C, F> {
    pub fn new(iter: A, get_second: F) -> Self {
        Self {
            iter: IterSwitch::Left(iter),
            get_second,
            found: Default::default(),
        }
    }
}

impl<
        A: Iterator<Item = Item>,
        B: Iterator<Item = Item>,
        C: Iterator<Item = Item>,
        Item,
        Env: Copy,
        F: FnMut(Env) -> B,
        G: FnMut(Env) -> C,
    > IterLazyChain3<A, B, C, Item, Env, F, G>
{
    pub fn new(iter: A, get_second: F, get_third: G, env: Env) -> Self {
        Self {
            iter: IterSwitch::Left(iter),
            env,
            get_second,
            get_third,
        }
    }
}

impl<T: Iterator<Item = V>, V: EnumSetType> Iterator for IterDistinct<T, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let value = self.iter.next()?;
            if self.existing.contains(value) {
                continue;
            }
            self.existing.insert(value);
            return Some(value);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Iterator<Item = V>, V> Iterator for IterOption<T, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(it) = &mut self.iter else {
            return None;
        };
        it.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Some(it) = &self.iter else {
            return (0, Some(0));
        };
        it.size_hint()
    }
}

impl<V: Copy, const N: usize> Iterator for IterSliceCopied<V, N> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }
        let i = self.index;
        self.index += 1;
        Some(self.slice[i])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len.saturating_sub(self.index), None)
    }
}

impl<A, B, C> Iterator for IterSwitch<A, B, C>
where
    A: Iterator<Item = C>,
    B: Iterator<Item = C>,
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterSwitch::Left(a) => a.next(),
            IterSwitch::Right(b) => b.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            IterSwitch::Left(a) => a.size_hint(),
            IterSwitch::Right(b) => b.size_hint(),
        }
    }
}

impl<A: Iterator<Item = C>, B: Iterator<Item = C>, C, F: Fn(bool) -> B> Iterator for IterLazyChain<A, B, C, F> {
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            IterSwitch::Left(a) => {
                let ret = a.next();
                if ret.is_some() {
                    self.found = true;
                    ret
                } else {
                    let mut b = (self.get_second)(self.found);
                    let ret = b.next();
                    self.iter = IterSwitch::Right(b);
                    ret
                }
            }
            IterSwitch::Right(b) => b.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.iter.size_hint().0, None)
    }
}

impl<
        A: Iterator<Item = Item>,
        B: Iterator<Item = Item>,
        C: Iterator<Item = Item>,
        Item,
        Env: Copy,
        F: FnMut(Env) -> B,
        G: FnMut(Env) -> C,
    > Iterator for IterLazyChain3<A, B, C, Item, Env, F, G>
{
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.iter {
                IterSwitch::Left(a) => {
                    let ret = a.next();
                    if ret.is_some() {
                        return ret;
                    } else {
                        self.iter = IterSwitch::Right(IterSwitch::Left((self.get_second)(self.env)));
                    }
                }
                IterSwitch::Right(IterSwitch::Left(b)) => {
                    let ret = b.next();
                    if ret.is_some() {
                        return ret;
                    } else {
                        self.iter = IterSwitch::Right(IterSwitch::Right((self.get_third)(self.env)));
                    }
                }
                IterSwitch::Right(IterSwitch::Right(c)) => {
                    let ret = c.next();
                    return if ret.is_some() { ret } else { None };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    const EMPTY: [usize; 0] = [];

    #[test]
    fn iter_slice_copied() {
        assert_eq!(
            Vec::<usize>::default(),
            IterSliceCopied::new(EMPTY, 0).collect::<Vec<_>>()
        );
        assert_eq!(vec![1usize], IterSliceCopied::new([1usize], 1).collect::<Vec<_>>());
        assert_eq!(
            vec![1usize, 2, 3, 4],
            IterSliceCopied::new([1usize, 2, 3, 4], 4).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![1usize, 2],
            IterSliceCopied::new([1usize, 2, 3, 4], 2).collect::<Vec<_>>()
        );
    }

    #[test]
    fn iter_option() {
        let empty_iter = EMPTY.iter().copied();
        assert_eq!(
            Vec::<usize>::default(),
            IterOption::<std::iter::Empty<usize>, usize>::new(None).collect::<Vec<usize>>()
        );
        assert_eq!(
            Vec::<usize>::default(),
            IterOption::new(Some(empty_iter)).collect::<Vec<usize>>()
        );
        assert_eq!(
            vec![1usize, 2],
            IterOption::new(Some([1usize, 2].iter().copied())).collect::<Vec<usize>>()
        );
    }

    #[test]
    fn iter_distinct() {
        use crate::status_impls::prelude::Element;

        assert_eq!(
            Vec::<Element>::default(),
            IterDistinct::new(EMPTY.iter().copied().map(|_| unreachable!())).collect::<Vec<Element>>()
        );
        assert_eq!(
            vec![Element::Pyro],
            IterDistinct::new([Element::Pyro].iter().copied()).collect::<Vec<_>>()
        );
        assert_eq!(
            vec![Element::Pyro, Element::Geo, Element::Cryo],
            IterDistinct::new(
                [
                    Element::Pyro,
                    Element::Geo,
                    Element::Geo,
                    Element::Cryo,
                    Element::Pyro,
                    Element::Cryo,
                    Element::Pyro
                ]
                .iter()
                .copied()
            )
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn iter_lazy_chain() {
        assert_eq!(
            Vec::<usize>::default(),
            IterLazyChain::new(EMPTY.iter().copied(), |_| EMPTY.iter().copied()).collect::<Vec<usize>>()
        );
        assert_eq!(
            vec![1usize, 2],
            IterLazyChain::new(EMPTY.iter().copied(), |_| [1usize, 2usize].iter().copied()).collect::<Vec<usize>>()
        );
        assert_eq!(
            Vec::<usize>::default(),
            IterLazyChain::new(EMPTY.iter().copied(), |has_elems| IterOption::new(
                has_elems.then_some([1usize, 2usize].iter().copied())
            ))
            .collect::<Vec<usize>>()
        );
        assert_eq!(
            vec![3usize, 4],
            IterLazyChain::new([3usize, 4].iter().copied(), |has_elems| IterOption::new(
                (!has_elems).then_some([1usize, 2].iter().copied())
            ),)
            .collect::<Vec<usize>>()
        );
        assert_eq!(
            vec![1usize, 2],
            IterLazyChain::new(EMPTY.iter().copied(), |has_elems| IterOption::new(
                (!has_elems).then_some([1usize, 2].iter().copied())
            ))
            .collect::<Vec<usize>>()
        );
        assert_eq!(
            vec![3usize, 4, 1, 2],
            IterLazyChain::new([3usize, 4].iter().copied(), |has_elems| IterOption::new(
                has_elems.then_some([1usize, 2].iter().copied())
            ))
            .collect::<Vec<usize>>()
        );
    }
}
