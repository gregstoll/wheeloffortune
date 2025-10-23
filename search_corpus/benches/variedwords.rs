use criterion::{criterion_group, criterion_main, Criterion};
use search_corpus::process_query_string;

fn small_word_many_options() {
    let result = process_query_string("mode=WheelOfFortune&pattern=???&absent_letters=xyzq").unwrap();
    assert_eq!("the", result[0]["word"].to_string());
}

fn small_word_few_options() {
    let result = process_query_string("mode=WheelOfFortune&pattern=t?e&absent_letters=r").unwrap();
    assert_eq!("the", result[0]["word"].to_string());
}

fn long_word_many_options() {
    let result = process_query_string("mode=WheelOfFortune&pattern=???????&absent_letters=xyzq").unwrap();
    assert_eq!("between", result[0]["word"].to_string());
}

fn long_word_few_options() {
    let result = process_query_string("mode=WheelOfFortune&pattern=?etwee?&absent_letters=xyzq").unwrap();
    assert_eq!("between", result[0]["word"].to_string());
}

fn longish_word_no_constraints_cryptogram() {
    let result = process_query_string("mode=Cryptogram&pattern=ABCDEF&absent_letters=").unwrap();
    assert_eq!("should", result[0]["word"].to_string());
}

fn longish_word_more_constraints_cryptogram() {
    let result = process_query_string("mode=Cryptogram&pattern=scABCD&absent_letters=").unwrap();
    assert_eq!("script", result[0]["word"].to_string());
}

fn small_word_many_options_criterion(c: &mut Criterion) {
    c.bench_function("small_word_many_options", |b| {
        b.iter(|| small_word_many_options())
    });
}

fn small_word_few_options_criterion(c: &mut Criterion) {
    c.bench_function("small_word_few_options", |b| {
        b.iter(|| small_word_few_options())
    });
}

fn long_word_few_options_criterion(c: &mut Criterion) {
    c.bench_function("long_word_few_options", |b| {
        b.iter(|| long_word_few_options())
    });
}

fn long_word_many_options_criterion(c: &mut Criterion) {
    c.bench_function("long_word_many_options", |b| {
        b.iter(|| long_word_many_options())
    });
}

fn longish_word_no_constraints_cryptogram_criterion(c: &mut Criterion) {
    c.bench_function("longish_word_no_constraints_cryptogram", |b| {
        b.iter(|| longish_word_no_constraints_cryptogram())
    });
}

fn longish_word_more_constraints_cryptogram_criterion(c: &mut Criterion) {
    c.bench_function("longish_word_more_constraints_cryptogram", |b| {
        b.iter(|| longish_word_more_constraints_cryptogram())
    });
}

criterion_group!(
    benches,
    small_word_many_options_criterion,
    small_word_few_options_criterion,
    long_word_many_options_criterion,
    long_word_few_options_criterion,
    longish_word_no_constraints_cryptogram_criterion,
    longish_word_more_constraints_cryptogram_criterion,
);
criterion_main!(benches);
