use criterion::{criterion_group, criterion_main, Criterion};
use search_corpus::process_query_string;


fn simulate_single() {
    process_query_string("pattern=t?e&absent_letters=").unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("variedwords", |b| b.iter(|| simulate_single()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);