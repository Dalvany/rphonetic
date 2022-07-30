use criterion::{criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

use rphonetic::*;

fn bench_encoder(c: &mut Criterion, encoder_name: &str, encoder: Box<dyn Encoder>, text: &str) {
    c.bench_function(encoder_name, |b| b.iter(|| encoder.encode(text)));
}

pub fn bench_caverphone_1(c: &mut Criterion) {
    let caverphone = Caverphone1;
    bench_encoder(c, "Caverphone 1", Box::new(caverphone), "Thompson");
}

pub fn bench_caverphone_2(c: &mut Criterion) {
    let caverphone = Caverphone2;
    bench_encoder(c, "Caverphone 2", Box::new(caverphone), "Thompson");
}

pub fn bench_cologne(c: &mut Criterion) {
    let cologne = Cologne;
    bench_encoder(c, "Cologne", Box::new(cologne), "m\u{00FC}ller")
}

pub fn bench_daitch_mokotoff_soundex_soundex(c: &mut Criterion) {
    let rules = include_str!("../rules/dmrules.txt");
    let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(rules)
        .build()
        .unwrap();
    // Do not use `bench_encoder` function as it will call `encode` and we want to bench also soundex (ie. with branching)
    c.bench_function("Daitch Mokotoff Soundex (soundex)", |b| {
        b.iter(|| daitch_mokotoff.soundex("Rosochowaciec"))
    });
}

pub fn bench_daitch_mokotoff_soundex_encode(c: &mut Criterion) {
    let rules = include_str!("../rules/dmrules.txt");
    let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(rules)
        .build()
        .unwrap();
    bench_encoder(
        c,
        "Daitch Mokotoff Soundex (encode)",
        Box::new(daitch_mokotoff),
        "Rosochowaciec",
    );
}

pub fn bench_double_metaphone(c: &mut Criterion) {
    let double_metaphone = DoubleMetaphone::default();
    bench_encoder(
        c,
        "Double Metaphone",
        Box::new(double_metaphone),
        "unconscious",
    );
}

pub fn bench_match_rating_approach(c: &mut Criterion) {
    let match_rating = MatchRatingApproach;
    bench_encoder(
        c,
        "Match Rating Approach",
        Box::new(match_rating),
        "Franciszek",
    );
}

pub fn bench_metaphone(c: &mut Criterion) {
    let metaphone = Metaphone::default();
    bench_encoder(c, "Metaphone", Box::new(metaphone), "Johnna");
}

pub fn bench_nysiis_strict(c: &mut Criterion) {
    let nysiis = Nysiis::default();
    bench_encoder(c, "Nysiis (strict)", Box::new(nysiis), "Phillipson");
}

pub fn bench_nysiis_not_strict(c: &mut Criterion) {
    let nysiis = Nysiis::new(false);
    bench_encoder(c, "Nysiis (not strict)", Box::new(nysiis), "Phillipson");
}

pub fn bench_refined_soundex(c: &mut Criterion) {
    let refined_soundex = RefinedSoundex::default();
    bench_encoder(
        c,
        "Refined Soundex",
        Box::new(refined_soundex),
        "Blotchet-Halls",
    );
}

pub fn bench_soundex(c: &mut Criterion) {
    let soundex = Soundex::default();
    bench_encoder(c, "Soundex", Box::new(soundex), "Blotchet-Halls");
}

pub fn bench_beider_morse(c: &mut Criterion) {
    let config_files = ConfigFiles::new(&PathBuf::from("./test_assets/cc-rules/")).unwrap();
    let builder = BeiderMorseBuilder::new(&config_files);
    let beider_morse = builder.build();
    c.bench_function("Beider-Morse", |b| b.iter(|| beider_morse.encode("Angelo")));
}

criterion_group!(
    name = caverphone;
    config = Criterion::default().sample_size(300);
    targets = bench_caverphone_1, bench_caverphone_2
);
criterion_group!(
    name = cologne;
    config = Criterion::default().sample_size(300);
    targets = bench_cologne
);
criterion_group!(
    name = daitch_mokotoff;
    config = Criterion::default().sample_size(300);
    targets = bench_daitch_mokotoff_soundex_soundex, bench_daitch_mokotoff_soundex_encode
);
criterion_group!(
    name = double_metaphone;
    config = Criterion::default().sample_size(300);
    targets = bench_double_metaphone
);
criterion_group!(
    name = match_rating_approach;
    config = Criterion::default().sample_size(300);
    targets = bench_match_rating_approach
);
criterion_group!(
    name = metaphone;
    config = Criterion::default().sample_size(300);
    targets = bench_metaphone
);
criterion_group!(
    name = nysiis;
    config = Criterion::default().sample_size(300);
    targets = bench_nysiis_strict, bench_nysiis_not_strict
);
criterion_group!(
    name = refined_soundex;
    config = Criterion::default().sample_size(300);
    targets = bench_refined_soundex
);
criterion_group!(
    name = soundex;
    config = Criterion::default().sample_size(300);
    targets = bench_soundex
);
criterion_group!(
    name = beider_morse;
    config = Criterion::default().sample_size(300);
    targets = bench_beider_morse
);

criterion_main!(
    caverphone,
    cologne,
    daitch_mokotoff,
    double_metaphone,
    match_rating_approach,
    metaphone,
    nysiis,
    refined_soundex,
    soundex,
    beider_morse,
);
