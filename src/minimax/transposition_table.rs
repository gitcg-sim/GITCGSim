use flurry::{HashMap, HashMapRef};

use crate::{data_structures::LinkedList, zobrist_hash::HashValue};

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
pub struct TTEntry<E: Sync + Send, A: Clone + Sync + Send> {
    pub flag: TTFlag,
    pub depth: u8,
    pub value: E,
    pub pv: LinkedList<A>,
}

impl<E: Sync + Send, A: Clone + Sync + Send> TTEntry<E, A> {
    #[inline]
    pub fn new(flag: TTFlag, depth: u8, value: E, pv: LinkedList<A>) -> Self {
        Self { flag, depth, value, pv }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TTKey(pub u64);

pub struct TT<E: Sync + Send, A: Clone + Sync + Send> {
    pub size: usize,
    pub table: HashMap<TTKey, TTEntry<E, A>>,
}

impl<E: Sync + Send, A: Clone + Sync + Send> std::fmt::Debug for TT<E, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TT").field("size", &self.size).finish()
    }
}

pub struct TTPin<'a, E: Sync + Send, A: Clone + Sync + Send> {
    pub tt: HashMapRef<'a, TTKey, TTEntry<E, A>>,
    pub size: usize,
}

impl<'a, E: Sync + Send, A: Clone + Sync + Send> TTPin<'a, E, A> {
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

pub const DEFAULT_SIZE_MB: u32 = 128;

impl<E: Sync + Send, A: Clone + Sync + Send> TT<E, A> {
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

impl<E: Sync + Send, A: Clone + Sync + Send> Default for TT<E, A> {
    fn default() -> Self {
        Self::new(DEFAULT_SIZE_MB)
    }
}
