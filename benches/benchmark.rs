use criterion::{black_box, criterion_group, criterion_main, Criterion};
use redust::{SharedStore, SharedStoreBase};

pub fn benchmark_set_get(c: &mut Criterion) {
    let store: SharedStore = SharedStore::new(false);

    let mut cmd_group = c.benchmark_group("cmd_group");

    cmd_group.significance_level(0.05).sample_size(500);

    cmd_group.bench_function("set", |b| {
        b.iter(|| {
            store.set(
                black_box("Test".to_string()),
                black_box(redust::DataType::String("Hello".to_string())),
                black_box(None),
                black_box(false),
                black_box(false),
            )
        })
    });

    cmd_group.bench_function("get", |b| {
        b.iter(|| store.get(black_box("Test".to_string())))
    });

    cmd_group.finish()
}

criterion_group!(benches, benchmark_set_get);
criterion_main!(benches);
