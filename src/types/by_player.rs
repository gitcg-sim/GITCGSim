use super::game_state::PlayerId;
use crate::std_subset::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByPlayer<T>(pub T, pub T);

impl<T> From<(T, T)> for ByPlayer<T> {
    #[inline]
    fn from((a, b): (T, T)) -> Self {
        Self(a, b)
    }
}

impl<T> From<ByPlayer<T>> for (T, T) {
    #[inline]
    fn from(value: ByPlayer<T>) -> Self {
        (value.0, value.1)
    }
}

impl<T> ByPlayer<T> {
    #[inline]
    pub const fn new(a: T, b: T) -> Self {
        Self(a, b)
    }

    /// Given function `f`, produces `ByPlayer::new(f(PlayerId::PlayerFirst), f(PlayerId::PlayerSecond))`
    #[inline]
    pub fn generate<F: FnMut(PlayerId) -> T>(mut f: F) -> Self {
        Self(f(PlayerId::PlayerFirst), f(PlayerId::PlayerSecond))
    }

    #[inline]
    pub fn get(&self, player_id: PlayerId) -> &T {
        match player_id {
            PlayerId::PlayerFirst => &self.0,
            PlayerId::PlayerSecond => &self.1,
        }
    }

    #[inline]
    pub fn get_mut(&mut self, player_id: PlayerId) -> &mut T {
        match player_id {
            PlayerId::PlayerFirst => &mut self.0,
            PlayerId::PlayerSecond => &mut self.1,
        }
    }

    pub fn map<A, F: FnMut(T) -> A>(self, mut f: F) -> ByPlayer<A> {
        ByPlayer::<A>::new(f(self.0), f(self.1))
    }

    pub fn for_each<F: FnMut(PlayerId, &T)>(&self, mut f: F) {
        f(PlayerId::PlayerFirst, &self.0);
        f(PlayerId::PlayerSecond, &self.1);
    }

    pub fn for_each_mut<F: FnMut(PlayerId, &mut T)>(&mut self, mut f: F) {
        f(PlayerId::PlayerFirst, &mut self.0);
        f(PlayerId::PlayerSecond, &mut self.1);
    }

    pub fn as_ref(&self) -> ByPlayer<&T> {
        ByPlayer(&self.0, &self.1)
    }
}

impl<T> Index<PlayerId> for ByPlayer<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: PlayerId) -> &Self::Output {
        self.get(index)
    }
}

impl<T> IndexMut<PlayerId> for ByPlayer<T> {
    #[inline]
    fn index_mut(&mut self, index: PlayerId) -> &mut Self::Output {
        self.get_mut(index)
    }
}
