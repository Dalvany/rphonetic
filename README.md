# Rust phonetic

This is a rust port of [Apache commons-codec](https://commons.apache.org/proper/commons-codec/)'s phonetic algorithms.

## Algorithms

Currently there are :
* [Caverphone 1](https://en.wikipedia.org/wiki/Caverphone)
* [Caverphone 2](https://en.wikipedia.org/wiki/Caverphone)

## Benchmarking

Benchmarking use [criterion](https://bheisler.github.io/criterion.rs/book/criterion_rs.html).

To run benches against `main` baseline :

```shell
cargo bench --bench benchmark -- --baseline main
```

To replace `main` baseline :

```shell
cargo bench --bench benchmark -- --save-baseline main
```

Do not run [Criterion benches on CI](https://bheisler.github.io/criterion.rs/book/faq.html#how-should-i-run-criterionrs-benchmarks-in-a-ci-pipeline).

