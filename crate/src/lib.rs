#[macro_use]
extern crate cfg_if;
extern crate nalgebra;
extern crate rand;
extern crate wasm_bindgen;

use nalgebra::{Point2, Vector3};
use rand::rngs::OsRng;
use rand::Rng;
use std::cmp::Ordering;
use std::slice;
use wasm_bindgen::prelude::*;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        // use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub struct Population {
  width: u32,
  height: u32,
  ref_values: Vec<u8>,
  members: Vec<RandomImage>,
  best_index: i32,
}

#[wasm_bindgen]
impl Population {
  pub fn new(width: u32, height: u32, ptr: *mut u8, reference_values_count: usize) -> Self {
    let ref_values = unsafe { slice::from_raw_parts(ptr, reference_values_count as usize) };
    Self {
      width,
      height,
      ref_values: ref_values.to_vec(),
      members: vec![],
      best_index: -1,
    }
  }

  pub fn evolve(&mut self) {
    let cull_percent = 0.2;

    self.members.sort();

    let cull_count = ((self.members.len() as f64) * cull_percent) as u8;
    for _ in 0..cull_count {
      self.members.remove(0);
    }
    for m in &mut self.members {
      m.mutate();
    }
    for _ in 0..cull_count {
      self.add_member();
    }

    // drop the worst-performers, mutate remaining, add new members
  }

  pub fn add_member(&mut self) {
    let mut member = RandomImage::new(self.width, self.height);
    member.calculate_fitness(&self.ref_values, 10, 10);
    self.members.push(member);
  }

  pub fn best_fitness(&mut self) -> f64 {
    if self.members.len() == 0 {
      0.0
    } else {
      self.members.sort();
      self.members.last().unwrap().fitness
    }
  }

  pub fn best_pixels(&mut self) -> *const Pixel {
    self.members.sort();
    self.members.last().unwrap().pixels()
  }
}

#[wasm_bindgen]
pub fn get_memory() -> JsValue {
  wasm_bindgen::memory()
}

#[derive(Clone, Debug)]
struct Color {
  r: u8,
  g: u8,
  b: u8,
  a: u8,
}

impl Default for Color {
  fn default() -> Self {
    Color::white()
  }
}

impl Color {
  fn from(p: &Pixel) -> Self {
    Self {
      r: p.r,
      g: p.g,
      b: p.b,
      a: p.a,
    }
  }

  fn random(rng: &mut rand::rngs::OsRng) -> Self {
    Self {
      r: rng.gen_range(0, 255),
      g: rng.gen_range(0, 255),
      b: rng.gen_range(0, 255),
      a: rng.gen_range(0, 255),
    }
  }

  fn white() -> Self {
    Self {
      r: 255,
      g: 255,
      b: 255,
      a: 255,
    }
  }

  // TODO: clean this up somehow
  fn add(&self, other: &Color) -> Color {
    let Color {
      r: o_r,
      g: o_g,
      b: o_b,
      a: o_a,
    } = other;
    let o_r = *o_r as f32;
    let o_g = *o_g as f32;
    let o_b = *o_b as f32;
    let o_a = (*o_a as f32) / 255.0;
    let r = self.r as f32;
    let g = self.g as f32;
    let b = self.b as f32;
    let a = (self.a as f32) / 255.0;
    let denom_part: f32 = a * (1.0 - o_a);
    let r = (o_r * o_a + r * denom_part) / (o_a + denom_part);
    let g = (o_g * o_a + g * denom_part) / (o_a + denom_part);
    let b = (o_b * o_a + b * denom_part) / (o_a + denom_part);
    let a = o_a + denom_part;
    Color {
      r: r as u8,
      g: g as u8,
      b: b as u8,
      a: (255.0 * a) as u8,
    }
  }
}

struct Triangle {
  v0: Point2<u32>,
  v1: Point2<u32>,
  v2: Point2<u32>,
  bbox: BBox,
}

#[derive(Clone)]
struct BBox {
  origin: Point2<u32>,
  extent: Point2<u32>,

  // For iterator
  _iter_x: u32,
  _iter_y: u32,
}

impl BBox {
  fn xmin(&self) -> u32 {
    self.origin.x
  }
  fn xmax(&self) -> u32 {
    self.extent.x
  }
  fn ymax(&self) -> u32 {
    self.extent.y
  }
  fn new(v0: &Point2<u32>, v1: &Point2<u32>, v2: &Point2<u32>) -> BBox {
    let xs = vec![v0.x, v1.x, v2.x];
    let ys = vec![v0.y, v1.y, v2.y];
    let &min_x = xs.iter().min().unwrap();
    let &min_y = ys.iter().min().unwrap();
    let &max_x = xs.iter().max().unwrap();
    let &max_y = ys.iter().max().unwrap();

    BBox {
      origin: Point2::new(min_x, min_y),
      extent: Point2::new(max_x, max_y),
      _iter_x: min_x,
      _iter_y: min_y,
    }
  }
}

impl Iterator for BBox {
  type Item = Point2<u32>;

  // Here, we define the sequence using `.curr` and `.next`.
  // The return type is `Option<T>`:
  //     * When the `Iterator` is finished, `None` is returned.
  //     * Otherwise, the next value is wrapped in `Some` and returned.
  fn next(&mut self) -> Option<Point2<u32>> {
    let cur = Point2::new(self._iter_x, self._iter_y);
    self._iter_x += 1;
    if self._iter_x >= self.xmax() {
      self._iter_x = self.xmin();
      self._iter_y += 1;
    }
    if self._iter_y >= self.ymax() {
      return None;
    }
    Some(cur)
  }
}

impl Triangle {
  pub fn new(v0x: u32, v0y: u32, v1x: u32, v1y: u32, v2x: u32, v2y: u32) -> Triangle {
    let v0 = Point2::new(v0x, v0y);
    let v1 = Point2::new(v1x, v1y);
    let v2 = Point2::new(v2x, v2y);
    let bbox = BBox::new(&v0, &v1, &v2);
    Triangle { v0, v1, v2, bbox }
  }

  /*
  Cross product
      [
      a[1] * b[2] - a[2] * b[1],
      a[2] * b[0] - a[0] * b[2],
      a[0] * b[1] - a[1] * b[0]
  ]
  */
  fn barycentric(&self, p: &Point2<u32>) -> Option<Vector3<f64>> {
    let v1 = Vector3::new(
      self.v2.x as f64 - self.v0.x as f64,
      self.v1.x as f64 - self.v0.x as f64,
      self.v0.x as f64 - p.x as f64,
    );
    let v2 = Vector3::new(
      self.v2.y as f64 - self.v0.y as f64,
      self.v1.y as f64 - self.v0.y as f64,
      self.v0.y as f64 - p.y as f64,
    );
    let u = Vector3::new(
      v1.y * v2.z - v1.z * v2.y,
      v1.z * v2.x - v1.x * v2.z,
      v1.x * v2.y - v1.y * v2.x,
    );
    if u.z.abs() < 1.0 {
      None
    } else {
      Some(Vector3::new(1.0 - (u.x + u.y) / u.z, u.y / u.z, u.x / u.z))
    }
  }

  fn contains(&self, p: &Point2<u32>) -> bool {
    match self.barycentric(p) {
      Some(v3) => v3.x > 0.0 && v3.y > 0.0 && v3.z > 0.0,
      _ => false,
    }
  }
}

#[derive(Clone)]
struct Gene(Point2<u32>, Point2<u32>, Point2<u32>, Color);
impl Gene {
  fn random(width: u32, height: u32, rng: &mut rand::rngs::OsRng) -> Gene {
    Gene(
      Point2::new(rng.gen_range(0, width), rng.gen_range(0, height)),
      Point2::new(rng.gen_range(0, width), rng.gen_range(0, height)),
      Point2::new(rng.gen_range(0, width), rng.gen_range(0, height)),
      Color::random(rng),
    )
  }
  fn mutate(&mut self, width: u32, height: u32, rng: &mut rand::rngs::OsRng) {
    let mutate_w = 0.2 * width as f64;
    let mutate_h = 0.2 * height as f64;

    fn clamp(v: f64, max: u32) -> u32 {
      if v < 0.0 {
        0
      } else if v as u32 >= max {
        max - 1
      } else {
        v as u32
      }
    }

    fn clamped_rand(v: u32, rnd_width: f64, max: u32, rng: &mut rand::rngs::OsRng) -> u32 {
      let v = v as f64;
      let lo = clamp((v - rnd_width / 2.0) as f64, max);
      let hi = clamp((v + rnd_width / 2.0) as f64, max);
      rng.gen_range(lo, hi) as u32
    }

    self.0.x = clamped_rand(self.0.x, mutate_w, width, rng);
    self.0.y = clamped_rand(self.0.y, mutate_h, height, rng);
    self.1.x = clamped_rand(self.1.x, mutate_w, width, rng);
    self.1.y = clamped_rand(self.1.y, mutate_h, height, rng);
    self.2.x = clamped_rand(self.2.x, mutate_w, width, rng);
    self.2.y = clamped_rand(self.2.y, mutate_h, height, rng);
  }
}

#[wasm_bindgen]
pub struct RandomImage {
  width: u32,
  height: u32,
  pixels: Vec<Pixel>,
  genes: Vec<Gene>,
  fitness: f64,
}

impl Ord for RandomImage {
  fn cmp(&self, other: &RandomImage) -> Ordering {
    if self.fitness > other.fitness {
      Ordering::Greater
    } else if self.fitness < other.fitness {
      Ordering::Less
    } else {
      Ordering::Equal
    }
  }
}

impl PartialOrd for RandomImage {
  fn partial_cmp(&self, other: &RandomImage) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for RandomImage {
  fn eq(&self, other: &RandomImage) -> bool {
    self.fitness == other.fitness
  }
}

impl Eq for RandomImage {}

#[wasm_bindgen]
impl RandomImage {
  pub fn new(width: u32, height: u32) -> RandomImage {
    let size = (width * height) as usize;
    let white = Color::white();
    let pixels: Vec<Pixel> = (0..size).map(|_| Pixel::of_color(&white)).collect();

    let gene_count = 10;
    let mut genes = vec![];
    let mut rng = OsRng::new().unwrap();
    for _ in 0..gene_count {
      genes.push(Gene::random(width, height, &mut rng));
    }
    RandomImage {
      width,
      height,
      pixels,
      genes,
      fitness: 0.0,
    }
  }

  pub fn render(&mut self) {
    for pixel in &mut self.pixels {
      pixel.set_color(&Color::white());
    }
    for gene in self.genes.clone() {
      let color = gene.3;
      self.triangle(
        gene.0.x, gene.0.y, gene.1.x, gene.1.y, gene.2.x, gene.2.y, &color,
      );
    }
  }

  fn re_render(&mut self) {
    for pixel in &mut self.pixels {
      pixel.set_color(&Color::white());
    }
    for gene in self.genes.clone() {
      let color = gene.3;
      self.triangle(
        gene.0.x, gene.0.y, gene.1.x, gene.1.y, gene.2.x, gene.2.y, &color,
      );
    }
  }

  pub fn mutate(&mut self) {
    let mut rng = OsRng::new().unwrap();

    for gene in &mut self.genes {
      gene.mutate(self.width, self.height, &mut rng);
    }
    self.re_render();
  }

  pub fn width(&self) -> u32 {
    self.width
  }

  pub fn height(&self) -> u32 {
    self.height
  }

  pub fn size(&self) -> usize {
    (self.width * self.height) as usize
  }

  pub fn pixels(&self) -> *const Pixel {
    self.pixels.as_ptr()
  }

  pub fn in_bounds(&self, x: i32, y: i32) -> bool {
    (x as u32) < self.width && x >= 0 && (y as u32) < self.height && y >= 0
  }

  fn pixel_index(&self, x: u32, y: u32) -> usize {
    let idx = (y * self.width + x) as usize;
    if idx >= self.size() {
      log!(
        "Tried to get pixel at ({},{}) -> idx {}, size {}",
        x,
        y,
        idx,
        self.size()
      );
    }
    idx
  }

  fn triangle(
    &mut self,
    v0x: u32,
    v0y: u32,
    v1x: u32,
    v1y: u32,
    v2x: u32,
    v2y: u32,
    color: &Color,
  ) {
    let t = Triangle::new(v0x, v0y, v1x, v1y, v2x, v2y);
    for p in t.bbox.clone() {
      if t.contains(&p) {
        let idx = self.pixel_index(p.x, p.y);
        self.pixels[idx].add_color(&color);
      }
    }
  }

  pub fn shrink(&self, width: u32, height: u32) -> RandomImage {
    self.shrink_via_tiling(width, height)
  }

  /*
  Shrinks an image to the supplied width & height.
  The algorithm is to determine tiles in the big image that map to pixels in the
  shrunk image, then average up the r,g,b values of all the pixels in each tile and
  assign the averages to a pixel in the shrunk image.
  I benchmarked this using Criterion and it appears to take ~50µs when shrinking a 500x500->100x100 image.
  But when compiled to WASM and run through the browser it takes ~60-70ms (a ~1000x slowdown).
  */
  pub fn shrink_via_tiling(&self, width: u32, height: u32) -> RandomImage {
    let tile_width = self.width / width;
    let tile_height = self.height / height;

    let size: usize = (tile_width * tile_height) as usize;
    let pixels: Vec<Pixel> = Vec::with_capacity(size);

    let mut shrunk_img = RandomImage {
      width: self.width / tile_width,
      height: self.height / tile_height,
      pixels,
      genes: vec![],
      fitness: 0.0,
    };

    for tile_row in 0..shrunk_img.height {
      for tile_col in 0..shrunk_img.width {
        let mut sum_r: u32 = 0;
        let mut sum_g: u32 = 0;
        let mut sum_b: u32 = 0;
        for x in (tile_col * tile_width)..((tile_col + 1) * tile_width) {
          for y in (tile_row * tile_height)..((tile_row + 1) * tile_height) {
            let pixel = self.get_pixel(x, y); // <-- This must copy the pixel, so it may be a source of slowdown,
                                              // but the pixel data structure should be fast to copy since it is just 4 u8s
            sum_r += pixel.r as u32;
            sum_g += pixel.g as u32;
            sum_b += pixel.b as u32;
          }
        }

        let avg_pixel = Pixel {
          r: (sum_r / size as u32) as u8,
          g: (sum_g / size as u32) as u8,
          b: (sum_b / size as u32) as u8,
          a: 255,
        };
        shrunk_img.pixels.push(avg_pixel);
      }
    }

    shrunk_img
  }

  pub fn calculate_fitness(
    &mut self,
    reference_values: &[u8],
    reference_w: u32,
    reference_h: u32,
  ) -> f64 {
    self.render();
    let shrunk = self.shrink(reference_w, reference_h);
    let fitness = shrunk.calculate_fitness_with_values(reference_values);
    self.fitness = fitness;
    fitness
  }

  pub fn calculate_fitness_with_values(&self, values: &[u8]) -> f64 {
    self.compare_values(values)
  }

  pub fn compare_values(&self, values: &[u8]) -> f64 {
    let mut err = 0.0;
    if self.pixels.len() * 4 != values.len() {
      log!("Got bad sizes for compare");
      panic!("Got bad sizes for compare");
    }

    fn squared_error(lhs: u8, rhs: u8) -> f64 {
      (lhs as f64 - rhs as f64).powi(2)
    }

    for (idx, pixel) in self.pixels.iter().enumerate() {
      err += squared_error(pixel.r, values[idx]);
      err += squared_error(pixel.g, values[idx + 1]);
      err += squared_error(pixel.b, values[idx + 2]);
      err += squared_error(pixel.a, values[idx + 3]);
    }

    let len = values.len() as f64;
    1.0 - (err / (len * len * 256.0 * 256.0))
  }

  // TODO - Is this a speed/memory issue, that the pixel is copied? It is a lightweight struct so
  // I would expect it to be fast. wasm-bindgen will not compile if this returns a ref to a pixel.
  pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
    self.pixels[self.pixel_index(x, y)]
  }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub struct Pixel {
  r: u8,
  g: u8,
  b: u8,
  a: u8,
}

#[wasm_bindgen]
impl Pixel {
  fn new() -> Pixel {
    Pixel {
      r: 255,
      g: 255,
      b: 255,
      a: 255,
    }
  }

  fn of_color(c: &Color) -> Pixel {
    Pixel {
      r: c.r,
      g: c.g,
      b: c.b,
      a: c.a,
    }
  }

  fn squared_error(&self, other: &Pixel) -> f64 {
    (self.r as f64 - other.r as f64).powi(2)
      + (self.g as f64 - other.g as f64).powi(2)
      + (self.b as f64 - other.b as f64).powi(2)
      + (self.a as f64 - other.a as f64).powi(2)
  }

  fn set_color(&mut self, c: &Color) {
    let Color { r, g, b, a } = c;
    self.r = *r;
    self.g = *g;
    self.b = *b;
    self.a = *a;
  }

  fn add_color(&mut self, c: &Color) {
    let new_color = Color::from(self).add(c);
    self.r = new_color.r;
    self.g = new_color.g;
    self.b = new_color.b;
    self.a = new_color.a;
  }
}
