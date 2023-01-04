use rand::prelude::*;
use serde::{Deserialize, Serialize};

/// Seedable RNG that can be serialized and deserialized.
#[repr(transparent)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "RngBytes")]
#[serde(into = "RngBytes")]
pub struct RngState(pub SmallRng);

impl RngCore for RngState {
    fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0.try_fill_bytes(dest)
    }
}

#[cfg(target_pointer_width = "64")]
type RngBytes = [u64; 4];
#[cfg(not(target_pointer_width = "64"))]
type RngBytes = [u32; 4];

// Here is the internal representation of SmallRng the rand-0.5.5 package
//
// #[cfg(target_pointer_width = "64")]
// type Rng = super::xoshiro256plusplus::Xoshiro256PlusPlus;
// #[cfg(not(target_pointer_width = "64"))]
// type Rng = super::xoshiro128plusplus::Xoshiro128PlusPlus;
//
// pub struct Xoshiro256PlusPlus { s: [u64; 4] }
// pub struct Xoshiro128PlusPlus { s: [u32; 4] }

impl RngState {
    #[inline]
    fn to_bytes(&self) -> &RngBytes {
        unsafe { std::mem::transmute::<&SmallRng, &RngBytes>(&self.0) }
    }

    #[inline]
    fn from_bytes(bytes: RngBytes) -> Self {
        Self(unsafe { std::mem::transmute::<RngBytes, SmallRng>(bytes) })
    }
}

impl From<RngBytes> for RngState {
    fn from(value: RngBytes) -> Self {
        Self::from_bytes(value)
    }
}

impl From<RngState> for RngBytes {
    fn from(value: RngState) -> Self {
        let bytes = value.to_bytes();
        *bytes
    }
}

impl From<SmallRng> for RngState {
    fn from(value: SmallRng) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use bincode;
    use serde_json;

    use rand::prelude::*;

    use super::RngState;

    #[test]
    fn test_bincode_serialize_deserialize_preserves_state() {
        for state in [0, 10, 123, 500] {
            let mut rng: RngState = SmallRng::seed_from_u64(state).into();
            let ser = bincode::serialize(&rng).unwrap();
            println!("Bytes = {ser:?}");
            let mut rng1: RngState = bincode::deserialize(&ser).unwrap();
            for _ in 0..100 {
                assert_eq!(rng.next_u64(), rng1.next_u64());
                assert_eq!(rng.next_u32(), rng1.next_u32());
            }
        }
    }

    #[test]
    fn test_json_serialize_deserialize_preserves_state() {
        for state in [0, 10, 123, 500] {
            let mut rng: RngState = SmallRng::seed_from_u64(state).into();
            let ser = serde_json::to_string_pretty(&rng).unwrap();
            println!("Bytes = {ser:?}");
            let mut rng1: RngState = serde_json::from_str(&ser).unwrap();
            for _ in 0..100 {
                assert_eq!(rng.next_u64(), rng1.next_u64());
                assert_eq!(rng.next_u32(), rng1.next_u32());
            }
        }
    }
}
