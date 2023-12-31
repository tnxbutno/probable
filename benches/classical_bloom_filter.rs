use criterion::{criterion_group, criterion_main, Criterion};
use probable::bloom_filters::{ClassicalBloomFilter, Filter};
use rand::distributions::Uniform;
use rand::prelude::IteratorRandom;
use rand::{thread_rng, Rng};
use std::collections::HashSet;

pub fn lookup_values(c: &mut Criterion) {
    let mut bf = ClassicalBloomFilter::new(10u32.pow(7), 0.02);
    let mut track_inserted = HashSet::new();

    let mut rng = thread_rng();
    let distribution = Uniform::new_inclusive(0, 10u64.pow(12));
    for _ in 0..10u32.pow(7) {
        let value = rng.sample(distribution).to_be_bytes();
        bf.insert(&value);
        track_inserted.insert(value);
    }

    let mut bgroup = c.benchmark_group("lookup-values");
    bgroup.bench_function("lookup-random-values", |b| {
        b.iter(|| bf.lookup(&rng.sample(distribution).to_be_bytes()))
    });

    bgroup.bench_function("lookup-inserted-values", |b| {
        b.iter(|| bf.lookup(track_inserted.iter().choose(&mut rng).unwrap()))
    });
}

criterion_group!(benches, lookup_values);
criterion_main!(benches);
