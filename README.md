[![Crate](https://img.shields.io/crates/v/rphonetic.svg)](https://crates.io/crates/rphonetic)
[![Build Status](https://github.com/Dalvany/rphonetic/actions/workflows/quality.yml/badge.svg)](https://github.com/Dalvany/rphonetic/actions/workflows/quality.yml)
[![codecov](https://codecov.io/gh/Dalvany/rphonetic/branch/main/graph/badge.svg)](https://codecov.io/gh/Dalvany/rphonetic)
[![dependency status](https://deps.rs/repo/github/Dalvany/rphonetic/status.svg)](https://deps.rs/repo/github/Dalvany/rphonetic)
[![Documentation](https://docs.rs/rphonetic/badge.svg)](https://docs.rs/rphonetic/)
[![Crate](https://img.shields.io/crates/d/rphonetic.svg)](https://crates.io/crates/rphonetic)
[![Crate](https://img.shields.io/crates/l/rphonetic.svg)](https://crates.io/crates/rphonetic)

# Rust phonetic

This is a rust port of v1.15 [Apache commons-codec](https://commons.apache.org/proper/commons-codec/)'s phonetic
algorithms.

## Algorithms

Currently, there are :

* [Beider-Morse](https://en.wikipedia.org/wiki/Daitch%E2%80%93Mokotoff_Soundex#Beider%E2%80%93Morse_Phonetic_Name_Matching_Algorithm)
* [Caverphone 1](https://en.wikipedia.org/wiki/Caverphone)
* [Caverphone 2](https://en.wikipedia.org/wiki/Caverphone)
* [Cologne](https://en.wikipedia.org/wiki/Cologne_phonetics)
* [Daitch Mokotoff Soundex](https://en.wikipedia.org/wiki/Daitch%E2%80%93Mokotoff_Soundex)
* [Match Rating Approach](https://en.wikipedia.org/wiki/Match_rating_approach)
* [Metaphone](https://en.wikipedia.org/wiki/Metaphone)
* [Metaphone (Double)](https://en.wikipedia.org/wiki/Metaphone#Double_Metaphone)
* [NYSIIS](https://en.wikipedia.org/wiki/New_York_State_Identification_and_Intelligence_System)
* [Phonex](https://citeseerx.ist.psu.edu/viewdoc/download;jsessionid=E3997DC51F2046A95EE6459F2B997029?doi=10.1.1.453.4046&rep=rep1&type=pdf)
* [Soundex](https://en.wikipedia.org/wiki/Soundex)
* [Soundex (Refined)](https://en.wikipedia.org/wiki/Soundex)

Please note that most of these algorithms are designed for the Latin alphabet, and they are usually designed for certain use cases (eg. 
English names / English dictionary words, ...etc.).

## Examples

### Beider-Morse

```rust
fn main() -> Result<(), rphonetic::PhoneticError> {
    use std::path::PathBuf;
    use rphonetic::{BeiderMorseBuilder, ConfigFiles, Encoder};

    let config_files = ConfigFiles::new(&PathBuf::from("./test_assets/cc-rules/"))?;
    let builder = BeiderMorseBuilder::new(&config_files);
    let beider_morse = builder.build();

    assert_eq!(beider_morse.encode("Van Helsing"),"(Ylznk|ilzn|ilznk|xilzn|xilznk)-(banilznk|bonilznk|fYnYlznk|fYnilznk|fanYlznk|fanilznk|fonYlznk|fonilznk|vYnYlznk|vYnilznk|vanYlznk|vaniilznk|vanilzn|vanilznk|vonYlznk|voniilznk|vonilzn|vonilznk)");
    Ok(())
}
```

### Caverphone 1 & 2

```rust
fn main() {
    use rphonetic::{Caverphone1, Encoder};

    let caverphone = Caverphone1;
    assert_eq!(caverphone.encode("Thompson"), "TMPSN1");
}
```

```rust
fn main() {
    use rphonetic::{Caverphone2, Encoder};

    let caverphone = Caverphone2;
    assert_eq!(caverphone.encode("Thompson"), "TMPSN11111");
}
```

### Cologne

```rust
fn main() {
    use rphonetic::{Cologne, Encoder};

    let cologne = Cologne;
    assert_eq!(cologne.encode("m\u{00FC}ller"), "657");
}
```

### Daitch-Mokotoff

```rust
fn main() -> Result<(), rphonetic::PhoneticError> {
    use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};

    const COMMONS_CODEC_RULES: &str = include_str!("./rules/dmrules.txt");

    let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
    assert_eq!(encoder.soundex("Rosochowaciec"), "944744|944745|944754|944755|945744|945745|945754|945755");
    Ok(())
}
```



### Match Rating Approach

```rust
fn main() {
    use rphonetic::{Encoder, MatchRatingApproach};
    
    let match_rating = MatchRatingApproach;
    assert_eq!(match_rating.encode("Smith"), "SMTH");
}
```

### Metaphone

```rust
fn main() {
    use rphonetic::{Encoder, Metaphone};
    
    let metaphone = Metaphone::default();
    assert_eq!(metaphone.encode("Joanne"), "JN");
}
```

### Metaphone (Double)

```rust
fn main() {
    use rphonetic::{DoubleMetaphone, Encoder};

    let double_metaphone = DoubleMetaphone::default();
    assert_eq!(double_metaphone.encode("jumped"), "JMPT");
    assert_eq!(double_metaphone.encode_alternate("jumped"), "AMPT");
}
```

### Phonex

```rust
fn main() {
    use rphonetic::{Phonex, Encoder};

    // Strict
    let phonex = Phonex::default();
    assert_eq!(phonex.encode("William"),"W450");
}
```

### Nysiis

```rust
fn main() {
    use rphonetic::{Nysiis, Encoder};

    // Strict
    let nysiis = Nysiis::default();
    assert_eq!(nysiis.encode("WESTERLUND"),"WASTAR");

    // Not strict
    let nysiis = Nysiis::new(false);
    assert_eq!(nysiis.encode("WESTERLUND"),"WASTARLAD");
}
```


### Soundex

```rust
fn main() {
    use rphonetic::{Encoder, Soundex};

    let soundex = Soundex::default();
    assert_eq!(soundex.encode("jumped"), "J513");
}
```

### Soundex (Refined)

```rust
fn main() {
    use rphonetic::{Encoder, RefinedSoundex};
    
    let refined_soundex = RefinedSoundex::default();
    assert_eq!(refined_soundex.encode("jumped"), "J408106");
}
```

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

