#[cfg(feature = "old_tt")]
use flurry::{HashMap, HashMapRef};

use crate::{linked_list::LinkedList, transposition_table::CacheTable};
use gitcg_sim::prelude::HashValue;

#[derive(Debug, Copy, Clone)]
pub enum TTFlag {
    /// Search raised alpha and was not pruned (PV-node)
    Exact,
    /// Search was beta pruned (CUT-node)
    Lower,
    /// Search did not raise alpha (ALL-node)
    Upper,
}

#[derive(Debug, Clone)]
pub struct TTEntry<E: Clone + Sync + Send, A: Clone + Sync + Send> {
    pub flag: TTFlag,
    pub depth: u8,
    pub value: E,
    pub pv: LinkedList<A>,
}

impl<E: Clone + Sync + Send, A: Clone + Sync + Send> TTEntry<E, A> {
    #[inline]
    pub fn new(flag: TTFlag, depth: u8, value: E, pv: LinkedList<A>) -> Self {
        Self { flag, depth, value, pv }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TTKey(pub HashValue);

impl From<TTKey> for usize {
    fn from(value: TTKey) -> Self {
        #[cfg(feature = "hash128")]
        {
            (value.0 % (1 + (usize::MAX as u128))) as usize
        }
        #[cfg(not(feature = "hash128"))]
        {
            value.0 as usize
        }
    }
}

pub struct TT<E: Clone + Sync + Send, A: Clone + Sync + Send> {
    #[cfg(feature = "old_tt")]
    pub size: usize,

    #[cfg(feature = "old_tt")]
    pub table: HashMap<TTKey, TTEntry<E, A>>,

    #[cfg(not(feature = "old_tt"))]
    pub table: CacheTable<TTKey, TTEntry<E, A>>,
}

impl<E: Clone + Sync + Send, A: Clone + Sync + Send> std::fmt::Debug for TT<E, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "old_tt")]
        {
            f.debug_struct("TT").field("size", &self.size).finish()
        }
        #[cfg(not(feature = "old_tt"))]
        f.debug_struct("TT")
            .field("megabytes", &self.table.get_megabytes())
            .finish()
    }
}

#[cfg(feature = "old_tt")]
pub struct TTPin<'a, E: Clone + Sync + Send, A: Clone + Sync + Send> {
    pub tt: HashMapRef<'a, TTKey, TTEntry<E, A>>,
    pub size: usize,
}

#[cfg(not(feature = "old_tt"))]
pub struct TTPin<'a, E: Clone + Sync + Send, A: Clone + Sync + Send> {
    pub tt: &'a CacheTable<TTKey, TTEntry<E, A>>,
}

#[cfg(feature = "old_tt")]
impl<'a, E: Clone + Sync + Send, A: Clone + Sync + Send> TTPin<'a, E, A> {
    #[inline]
    pub fn new(tt: HashMapRef<'a, TTKey, TTEntry<E, A>>, size: usize) -> Self {
        Self { tt, size }
    }

    #[inline]
    pub fn get(&self, key: &TTKey) -> Option<&TTEntry<E, A>> {
        self.tt.get(key)
    }

    #[inline]
    pub fn insert(&self, key: TTKey, entry: TTEntry<E, A>) {
        let n = self.tt.len();
        if n >= self.size {
            let mut d = 1;
            let target_size = 7 * self.size / 10;
            while self.tt.len() - 4 >= target_size {
                self.tt.retain(|_, e| e.depth >= d);
                d += 1;
            }
        }
        self.tt.insert(key, entry);
    }
}

#[cfg(not(feature = "old_tt"))]
impl<'a, E: Clone + Sync + Send, A: Clone + Sync + Send> TTPin<'a, E, A> {
    #[inline]
    pub fn new(tt: &'a CacheTable<TTKey, TTEntry<E, A>>) -> Self {
        Self { tt }
    }

    #[inline]
    pub fn get(&self, key: &TTKey) -> Option<TTEntry<E, A>> {
        self.tt.get(key)
    }

    #[inline]
    pub fn insert(&self, key: TTKey, entry: TTEntry<E, A>) {
        let depth = entry.depth;
        self.tt.replace_if(&key, entry, |entry1| entry1.depth < depth);
    }
}

pub const DEFAULT_SIZE_MB: u32 = 128;

#[cfg(feature = "old_tt")]
impl<E: Clone + Sync + Send, A: Clone + Sync + Send> TT<E, A> {
    #[inline]
    pub fn to_key(hash: HashValue) -> TTKey {
        TTKey(hash)
    }

    pub fn new(size_mb: u32) -> Self {
        let table = HashMap::new();
        let size_bytes: usize = (size_mb as usize) * (1024 * 1024);
        let size = size_bytes / (std::mem::size_of::<TTKey>() + std::mem::size_of::<TTEntry<E, A>>());
        table.pin().reserve(size);
        Self { size, table }
    }

    #[inline]
    pub fn pin(&self) -> TTPin<E, A> {
        TTPin::new(self.table.pin(), self.size)
    }
}

#[cfg(not(feature = "old_tt"))]
impl<E: Clone + Sync + Send, A: Clone + Sync + Send> TT<E, A> {
    #[inline]
    pub fn to_key(hash: HashValue) -> TTKey {
        TTKey(hash)
    }

    pub fn new(size_mb: u32) -> Self {
        Self {
            table: CacheTable::new(size_mb as usize),
        }
    }

    #[inline]
    pub fn pin(&self) -> TTPin<E, A> {
        TTPin::new(&self.table)
    }
}

impl<E: Clone + Sync + Send, A: Clone + Sync + Send> Default for TT<E, A> {
    fn default() -> Self {
        Self::new(DEFAULT_SIZE_MB)
    }
}
