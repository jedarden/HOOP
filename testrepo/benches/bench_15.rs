//! Benchmark 15 - Performance tests

use std::collections::HashMap;

#[bench]
fn bench_hashmap_insert_15(b: &mut test::Bencher) {
    b.iter(|| {
        let mut map = HashMap::new();
        for j in 0..100 {
            map.insert(j, j * 2);
        }
    });
}
