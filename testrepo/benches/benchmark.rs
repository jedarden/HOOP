// Simple benchmark stub for testrepo fixture
// This is a minimal benchmark to satisfy Cargo.toml

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn simple_benchmark(c: &mut Criterion) {
    c.bench_function("simple_operation", |b| {
        b.iter(|| {
            // Simple operation for benchmarking
            let result = black_box(2 + 2);
            result
        })
    });
}

criterion_group!(benches, simple_benchmark);
criterion_main!(benches);
