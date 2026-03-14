use swiftlet::load_grammar::get_terminals;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_get_terminals(c: &mut Criterion) {
    c.bench_function("get_terminals", |b| {
        b.iter(|| {
            get_terminals();
        })
    });
}

criterion_group!(benches, bench_get_terminals);
criterion_main!(benches);
