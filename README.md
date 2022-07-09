# Rust phonetic

This is a rust port of v1.15 [Apache commons-codec](https://commons.apache.org/proper/commons-codec/)'s phonetic
algorithms.

## Algorithms

Currently, there are :

* [Caverphone 1](https://en.wikipedia.org/wiki/Caverphone)
* [Caverphone 2](https://en.wikipedia.org/wiki/Caverphone)
* [Cologne](https://en.wikipedia.org/wiki/Cologne_phonetics)
* [Daitch Mokotoff Soundex](https://en.wikipedia.org/wiki/Daitch%E2%80%93Mokotoff_Soundex)
* [Double Metaphone](https://en.wikipedia.org/wiki/Metaphone#Double_Metaphone)
* [Match Rating Approach](https://en.wikipedia.org/wiki/Match_rating_approach)
* [Metaphone](https://en.wikipedia.org/wiki/Metaphone)
* [NYSIIS](https://en.wikipedia.org/wiki/New_York_State_Identification_and_Intelligence_System)
* [Refined Soundex](https://en.wikipedia.org/wiki/Soundex)

Please note that most of these algorithms are design for ASCII and there are usually design for certain use case (eg.
english names, ...etc).

## Benchmarking

Benchmarking use [criterion](https://bheisler.github.io/criterion.rs/book/criterion_rs.html).

They were done on an Intel® Core™ i7-4720HQ with 16GB RAM.

To run benches against `main` baseline :

```shell
cargo bench --bench benchmark -- --baseline main
```

To replace `main` baseline :

```shell
cargo bench --bench benchmark -- --save-baseline main
```

Do not
run [Criterion benches on CI](https://bheisler.github.io/criterion.rs/book/faq.html#how-should-i-run-criterionrs-benchmarks-in-a-ci-pipeline)
.

