use barat::load_grammar::get_terminals;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_get_terminals(c: &mut Criterion) {
    c.bench_function("get_terminals", |b| b.iter(|| {
        get_terminals();
    }));
}

criterion_group!(benches, bench_get_terminals);
criterion_main!(benches);
