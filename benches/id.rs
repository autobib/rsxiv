use rsxiv::id::ArticleId;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("old-id", |b| {
        b.iter(|| ArticleId::parse(black_box("hep-th/0109001")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
