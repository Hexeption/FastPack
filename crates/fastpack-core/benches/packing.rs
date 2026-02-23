use criterion::{Criterion, criterion_group, criterion_main};

fn bench_placeholder(c: &mut Criterion) {
    c.bench_function("maxrects_placeholder", |b| b.iter(|| 42_u32));
}

criterion_group!(benches, bench_placeholder);
criterion_main!(benches);
