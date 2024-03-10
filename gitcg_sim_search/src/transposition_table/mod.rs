use std::sync::RwLock;

pub const ENTRIES_PER_BUCKET: usize = 32;

pub type Bucket<K, V> = [Option<(K, V)>; ENTRIES_PER_BUCKET];

/// A shared hash table designed for transposition tables.
/// Features:
///  - Bucket-based concurrent access
///  - Allocate memory based on specified number of megabytes
///  - Replace-if based on a function
pub struct CacheTable<K: Eq + Copy + Into<usize>, V: Sized + Clone> {
    megabytes: usize,
    number_of_entries: usize,
    bucket_count: usize,
    buckets: Vec<RwLock<Bucket<K, V>>>,
}

impl<K: Eq + Copy + Into<usize>, V: Sized + Clone> std::fmt::Debug for CacheTable<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheTable")
            .field("megabytes", &self.megabytes)
            .field("number_of_entries", &self.number_of_entries)
            .field("bucket_count", &self.bucket_count)
            .field("occupancy", &self.occupancy())
            .finish()
    }
}

impl<K: Eq + Copy + Into<usize>, V: Sized + Clone> CacheTable<K, V> {
    pub fn new(megabytes: usize) -> Self {
        let bytes = megabytes * 1024 * 1024;
        let bytes_per_bucket = std::mem::size_of::<RwLock<Bucket<K, V>>>();
        let bucket_count = bytes / bytes_per_bucket;
        let number_of_entries = bucket_count * ENTRIES_PER_BUCKET;
        let bucket_count = number_of_entries / ENTRIES_PER_BUCKET;
        let buckets = (0..bucket_count).map(|_| Default::default()).collect();
        Self {
            megabytes,
            number_of_entries,
            bucket_count,
            buckets,
        }
    }

    pub fn megabytes(&self) -> usize {
        self.megabytes
    }

    pub fn max_entries(&self) -> usize {
        self.number_of_entries
    }

    pub fn bucket_count(&self) -> usize {
        self.bucket_count
    }

    pub fn occupied_count(&self) -> usize {
        self.buckets
            .iter()
            .map(|bucket| bucket.read().expect("count").iter().filter(|x| x.is_some()).count())
            .sum()
    }

    pub fn occupancy(&self) -> f64 {
        (self.occupied_count() as f64) / (self.number_of_entries as f64)
    }

    #[inline]
    fn decompose(&self, k: &K) -> (usize, usize) {
        let index = std::convert::Into::<usize>::into(*k) % self.number_of_entries;
        // (bucket_index, entry_index)
        (index / ENTRIES_PER_BUCKET, index % ENTRIES_PER_BUCKET)
    }

    pub fn clear(&self) {
        for b in &self.buckets {
            let Ok(mut bucket) = b.write() else { continue };
            bucket.iter_mut().for_each(|r| *r = None)
        }
    }

    pub fn get(&self, k: &K) -> Option<V> {
        if self.number_of_entries == 0 {
            return None;
        }
        let (bi, ei) = self.decompose(k);
        let Ok(bucket) = self.buckets[bi].read() else {
            return None;
        };
        let (k1, v) = bucket[ei].as_ref()?;
        if k1 == k {
            Some(v.clone())
        } else {
            None
        }
    }

    pub fn set(&self, k: &K, v: V) {
        if self.number_of_entries == 0 {
            return;
        }
        let (bi, ei) = self.decompose(k);
        let Ok(mut bucket) = self.buckets[bi].write() else {
            return;
        };
        bucket[ei] = Some((*k, v));
    }

    pub fn replace_if<F: Fn(&V) -> bool>(&self, k: &K, v: V, should_replace: F) -> bool {
        if self.number_of_entries == 0 {
            return false;
        }
        let (bi, ei) = self.decompose(k);
        let Ok(mut bucket) = self.buckets[bi].write() else {
            return false;
        };
        let entry_ref = &mut bucket[ei];
        let Some((k0, v0)) = entry_ref else {
            // Case 1: already empty
            *entry_ref = Some((*k, v));
            return true;
        };

        if !should_replace(v0) {
            // Case 2: should not replace
            return false;
        }

        // Case 3: should replace
        *k0 = *k;
        *v0 = v;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::CacheTable;

    #[test]
    fn test_get_set() {
        let table = CacheTable::<usize, u8>::new(1);
        assert_eq!(0, table.occupied_count());
        let k1 = 123usize;
        let v1 = 100u8;
        let k2 = 10usize;
        let v2 = 40u8;
        let v3 = 5u8;

        table.set(&k1, v1);
        assert_eq!(Some(v1), table.get(&k1));
        assert_eq!(None, table.get(&k2));

        table.set(&k1, v2);
        assert_eq!(Some(v2), table.get(&k1));
        assert_eq!(1, table.occupied_count());

        table.set(&k2, v3);
        assert_eq!(Some(v3), table.get(&k2));
    }

    #[test]
    fn test_replace_if() {
        let table = CacheTable::<usize, u8>::new(1);
        assert_eq!(0, table.occupied_count());
        let k1 = 123usize;
        let v1 = 14u8;
        let v2 = 50u8;

        table.set(&k1, v1);
        table.replace_if(&k1, v2, |prev_value| *prev_value >= 50u8);
        assert_eq!(Some(v1), table.get(&k1));
        assert_eq!(1, table.occupied_count());

        table.replace_if(&k1, v2, |prev_value| *prev_value >= 40u8);
        assert_eq!(Some(v1), table.get(&k1));
        assert_eq!(1, table.occupied_count());

        table.replace_if(&k1, v2, |prev_value| *prev_value >= 10u8);
        assert_eq!(Some(v2), table.get(&k1));
        assert_eq!(1, table.occupied_count());
    }
}
