use criterion::{criterion_group, criterion_main, Criterion};
use search_corpus::process_query_string;


fn small_word_many_options() {
    let result = process_query_string("pattern=???&absent_letters=xyzq").unwrap();
    assert_eq!("the", result[0]["word"].to_string());
}

fn small_word_few_options() {
    let result = process_query_string("pattern=t?e&absent_letters=r").unwrap();
    assert_eq!("the", result[0]["word"].to_string());
}

fn long_word_many_options() {
    let result = process_query_string("pattern=???????&absent_letters=xyzq").unwrap();
    assert_eq!("between", result[0]["word"].to_string());
}

fn long_word_few_options() {
    let result = process_query_string("pattern=?etwee?&absent_letters=xyzq").unwrap();
    assert_eq!("between", result[0]["word"].to_string());
}

fn small_word_many_options_criterion(c: &mut Criterion) {
    c.bench_function("small_word_many_options", |b| b.iter(|| small_word_many_options()));
}

fn small_word_few_options_criterion(c: &mut Criterion) {
    c.bench_function("small_word_few_options", |b| b.iter(|| small_word_few_options()));
}

fn long_word_many_options_criterion(c: &mut Criterion) {
    c.bench_function("long_word_many_options", |b| b.iter(|| long_word_many_options()));
}

fn long_word_few_options_criterion(c: &mut Criterion) {
    c.bench_function("long_word_few_options", |b| b.iter(|| long_word_few_options()));
}

criterion_group!(benches, small_word_many_options_criterion, small_word_few_options_criterion, long_word_many_options_criterion, long_word_few_options_criterion);
criterion_main!(benches);