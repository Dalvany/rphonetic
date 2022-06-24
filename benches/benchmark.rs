use criterion::{criterion_group, criterion_main, Criterion};
use rphonetic::*;

fn bench_encoder(c: &mut Criterion, encoder_name:&str, encoder:Box<dyn Encoder>, text:&str) {
    c.bench_function(encoder_name, |b| b.iter(|| encoder.encode(text)));
}

pub fn bench_caverphone_1(c: &mut Criterion) {
    let caverphone = Caverphone1::new();
    bench_encoder(c, "Caverphone 1",Box::new(caverphone), "Thompson");
}

pub fn bench_caverphone_2(c: &mut Criterion) {
    let caverphone = Caverphone1::new();
    bench_encoder(c, "Caverphone 2",Box::new(caverphone), "Thompson");
}

pub fn bench_cologne(c: &mut Criterion) {
    let cologne = Cologne;
    bench_encoder(c, "Cologne", Box::new(cologne), "m\u{00FC}ller")
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
criterion_main!(caverphone, cologne);