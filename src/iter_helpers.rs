/// Iterate through one of 2 different iterators with the same `Item`.
pub enum IterSwitch<A: Iterator<Item = C>, B: Iterator<Item = C>, C> {
    Left(A),
    Right(B),
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

    fn count(self) -> usize {
        match self {
            IterSwitch::Left(a) => a.count(),
            IterSwitch::Right(b) => b.count(),
        }
    }
}
