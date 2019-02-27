#[macro_use]
extern crate cfg_if;
extern crate nalgebra;
extern crate rand;
extern crate wasm_bindgen;

use nalgebra::{Point2, Vector3};
use rand::rngs::OsRng;
use rand::Rng;
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
pub fn get_memory() -> JsValue {
  wasm_bindgen::memory()
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

#[derive(Debug, Clone)]
struct Segment {
  x0: u32,
  x1: u32,
  y0: u32,
  y1: u32,
}
impl Segment {
  fn random(width: u32, height: u32, rng: &mut rand::rngs::OsRng) -> Segment {
    let x0 = rng.gen_range(0, width) as u32;
    let x1 = rng.gen_range(0, width) as u32;
    let y0 = rng.gen_range(0, height) as u32;
    let y1 = rng.gen_range(0, height) as u32;
    Segment { x0, x1, y0, y1 }
  }

  fn mutate(&mut self, width: u32, height: u32, rng: &mut rand::rngs::OsRng) {
    let mutate_w = 0.1 * width as f64;
    let mutate_h = 0.1 * height as f64;

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

    self.x0 = clamped_rand(self.x0, mutate_w, width, rng);
    self.x1 = clamped_rand(self.x1, mutate_w, width, rng);
    self.y0 = clamped_rand(self.y0, mutate_h, height, rng);
    self.y1 = clamped_rand(self.y1, mutate_h, height, rng);
  }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct RandomImage {
  width: u32,
  height: u32,
  pixels: Vec<Pixel>,
  segments: Vec<Segment>,
}

#[wasm_bindgen]
impl RandomImage {
  pub fn reset(&mut self) {
    self.segments.clear();

    let segment_count = 10;
    let mut rng = OsRng::new().unwrap();
    for _ in 0..segment_count {
      self
        .segments
        .push(Segment::random(self.width, self.height, &mut rng));
    }
    self.re_render();
  }

  pub fn new(width: u32, height: u32) -> RandomImage {
    let size = (width * height) as usize;
    let pixels: Vec<Pixel> = (0..size).map(|_| Pixel::new()).collect();

    let segment_count = 10;
    let mut segments = vec![];
    let mut rng = OsRng::new().unwrap();
    for _ in 0..segment_count {
      segments.push(Segment::random(width, height, &mut rng));
    }
    let mut ri = RandomImage {
      width,
      height,
      pixels,
      segments,
    };
    for segment in ri.segments.clone() {
      ri.line(
        segment.x0 as i32,
        segment.y0 as i32,
        segment.x1 as i32,
        segment.y1 as i32,
      );
    }
    ri
  }

  fn re_render(&mut self) {
    for pixel in &mut self.pixels {
      pixel.r = 255;
      pixel.g = 255;
      pixel.b = 255;
      pixel.a = 255;
    }
    for segment in self.segments.clone() {
      self.line(
        segment.x0 as i32,
        segment.y0 as i32,
        segment.x1 as i32,
        segment.y1 as i32,
      );
    }
  }

  pub fn mutate(&mut self) {
    let mut rng = OsRng::new().unwrap();

    for segment in &mut self.segments {
      segment.mutate(self.width, self.height, &mut rng);
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

  pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
    if y0 == y1 {
      return self._horiz_line(y0, x0, x1);
    } else if x0 == x1 {
      return self._vert_line(x0, y0, y1);
    }
    if x0 > x1 {
      // re-order so that we go from left to right
      return self.line(x1, y1, x0, y0);
    }
    let delta_x = x1 - x0;
    let delta_y = y1 - y0;
    let delta_err: f32 = (delta_y as f32 / delta_x as f32).abs();
    let mut error: f32 = 0.0;
    let mut y = y0;
    for x in x0..x1 {
      if self.in_bounds(x, y) {
        let index = self.pixel_index(x as u32, y as u32);
        self.pixels[index].add_color();
      }

      error += delta_err;
      if error >= 0.5 {
        if delta_y > 0 {
          y = y + 1;
        } else {
          y = y - 1;
        }
        error = error - 1.0;
      }
    }
  }

  fn _vert_line(&mut self, x0: i32, y0: i32, y1: i32) {
    let x = x0;
    for y in y0..y1 {
      if self.in_bounds(x, y) {
        let index = self.pixel_index(x as u32, y as u32);
        self.pixels[index].add_color();
      }
    }
  }

  fn _horiz_line(&mut self, y0: i32, x0: i32, x1: i32) {
    let y = y0;
    for x in x0..x1 {
      if self.in_bounds(x, y) {
        let index = self.pixel_index(x as u32, y as u32);
        self.pixels[index].add_color();
      }
    }
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

  pub fn triangle(&mut self, v0x: u32, v0y: u32, v1x: u32, v1y: u32, v2x: u32, v2y: u32) {
    let t = Triangle::new(v0x, v0y, v1x, v1y, v2x, v2y);
    for p in t.bbox.clone() {
      if t.contains(&p) {
        let idx = self.pixel_index(p.x, p.y);
        self.pixels[idx].r = 0;
        self.pixels[idx].g = 0;
        self.pixels[idx].b = 0;
        self.pixels[idx].a = 255;
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
      segments: vec![],
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

  pub fn compare(&self, other: &RandomImage) -> f64 {
    let mut err = 0.0;
    if self.size() != other.size() {
      log!(
        "Got bad sizes for compare {} <> {}",
        self.size(),
        other.size()
      );
      panic!("Got bad sizes for compare");
    }
    for (lhs, rhs) in self.pixels.iter().zip(&other.pixels) {
      err += lhs.squared_error(&rhs);
    }
    err / self.pixels.len() as f64
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
  pub r: u8,
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

  fn squared_error(&self, other: &Pixel) -> f64 {
    (self.r as f64 - other.r as f64).powi(2)
      + (self.g as f64 - other.g as f64).powi(2)
      + (self.b as f64 - other.b as f64).powi(2)
      + (self.a as f64 - other.a as f64).powi(2)
  }

  pub fn add_color(&mut self) {
    self.r = 0;
    self.g = 0;
    self.b = 0;
  }
}
