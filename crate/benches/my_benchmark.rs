#[macro_use]
extern crate criterion;
extern crate rust_webpack;

use criterion::Criterion;
use rust_webpack::RandomImage;

fn criterion_benchmark(c: &mut Criterion) {
  let img = RandomImage::new(100, 100);
  c.bench_function("basic shrink 100x100 to 10x10", move |b| {
    b.iter(|| {
      img.shrink(10, 10);
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
