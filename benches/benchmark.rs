use criterion::{criterion_group, criterion_main, Criterion};
use rphonetic::*;

fn bench_encoder(c: &mut Criterion, encoder_name:&str, encoder:Box<dyn Encoder>) {
    c.bench_function(encoder_name, |b| b.iter(|| encoder.encode("test")));
}

pub fn bench_caverphone_1(c: &mut Criterion) {
    let caverphone = Caverphone1::new();
    bench_encoder(c, "Caverphone 1",Box::new(caverphone));
}

pub fn bench_caverphone_2(c: &mut Criterion) {
    let caverphone = Caverphone1::new();
    bench_encoder(c, "Caverphone 2",Box::new(caverphone));
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(200);
    targets = bench_caverphone_1, bench_caverphone_2
);
criterion_main!(benches);