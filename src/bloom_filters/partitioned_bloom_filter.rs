use crate::bloom_filters::base::Filter;
use bit_vec::BitVec;
use xxhash_rust::xxh3::xxh3_64_with_seed;

pub struct PartitionedBloomFilter {
    /// number of hash functions
    k: u64,

    partition_size: usize,
    partitions: Vec<BitVec>,
}

impl Filter for PartitionedBloomFilter {
    /// n -- number of elements to insert
    /// f -- the false positive rate
    fn new(n: u32, f: f64) -> Self {
        let m = Self::calculate_m(f, n);
        let k = Self::calculate_k(m, n);
        let partition_size = (m / k) as usize;
        Self {
            k,
            partition_size,
            partitions: std::iter::repeat(BitVec::from_elem(partition_size, false))
                .take(k as usize)
                .collect(),
        }
    }

    fn insert(&mut self, value: &[u8]) {
        let hash1 = xxh3_64_with_seed(value, 0) % self.partition_size as u64;
        let hash2 = xxh3_64_with_seed(value, 64) % self.partition_size as u64;
        for i in 0..self.k {
            let idx = ((hash1 + i * hash2) % self.partition_size as u64) as usize;
            self.partitions[i as usize].set(idx, true);
        }
    }

    fn lookup(&self, value: &[u8]) -> bool {
        let hash1 = xxh3_64_with_seed(value, 0) % self.partition_size as u64;
        let hash2 = xxh3_64_with_seed(value, 64) % self.partition_size as u64;
        for i in 0..self.k {
            let idx = ((hash1 + i * hash2) % self.partition_size as u64) as usize;
            if self.partitions[i as usize].get(idx) == Some(false) {
                return false;
            }
        }
        true
    }

    fn get_size(&self) -> usize {
        self.partitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Uniform;
    use rand::{thread_rng, Rng};
    use std::collections::HashSet;

    #[test]
    fn partitioned_simple_check() {
        let mut bf = PartitionedBloomFilter::new(10, 0.01);
        bf.insert(&1u32.to_be_bytes());
        bf.insert(&10u32.to_be_bytes());
        bf.insert(&30u32.to_be_bytes());

        let res = bf.lookup(&1u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&10u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&30u32.to_be_bytes());
        assert!(res, "stored value is not found!");

        let res = bf.lookup(&45u32.to_be_bytes());
        assert!(!res, "not stored value is found!");
    }

    #[test]
    fn verify_partitioned_bf_false_positive_rate() {
        let mut bf = PartitionedBloomFilter::new(10u32.pow(7), 0.02);
        let mut track_inserted = HashSet::new();

        let mut rng = thread_rng();
        let distribution = Uniform::new_inclusive(0, 10u64.pow(12));
        for _ in 0..10u32.pow(7) {
            let value = rng.sample(distribution).to_be_bytes();
            bf.insert(&value);
            track_inserted.insert(value);
        }

        let mut false_positive = 0;
        for _ in 0..10u32.pow(6) {
            let value = rng.sample(distribution).to_be_bytes();
            let found = bf.lookup(&value);
            if found && track_inserted.get(&value).is_none() {
                false_positive += 1;
            }
        }

        dbg!("partitioned", false_positive);
        // check that false positive rate is ~2%
        assert!(19900 < false_positive && false_positive < 21000);
    }
}
