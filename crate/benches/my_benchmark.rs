#[macro_use]
extern crate criterion;
extern crate image_evol;

use criterion::Criterion;
use image_evol::RandomImage;

/*
 benchmark results on 2/9/19

 basic shrink via hashmap 500x500 to 100x100
                        time:   [11.711 ms 11.752 ms 11.799 ms]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

basic shrink via tiling 500x500 to 100x100
                        time:   [806.91 us 813.14 us 819.63 us]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

*/

// 2/19/19 bench: 813µs
fn bench_shrink_tiling(c: &mut Criterion) {
  let img = RandomImage::new(500, 500);
  c.bench_function("basic shrink via tiling 500x500 to 100x100", move |b| {
    b.iter(|| {
      img.shrink_via_tiling(100, 100);
    })
  });
}

// 2/19/19 bench: 11.752ms
// Removed this fn on 2/26/19
//
// fn bench_shrink_hashmap(c: &mut Criterion) {
//   let img = RandomImage::new(500, 500);
//   c.bench_function("basic shrink via hashmap 500x500 to 100x100", move |b| {
//     b.iter(|| {
//       img.shrink_via_hashmap(100, 100);
//     })
//   });
// }

criterion_group!(benches, bench_shrink_tiling);
criterion_main!(benches);
