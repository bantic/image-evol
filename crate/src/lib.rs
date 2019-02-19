#[macro_use]
extern crate cfg_if;
extern crate rand;
extern crate wasm_bindgen;

use rand::rngs::OsRng;
use rand::Rng;
use std::collections::HashMap;
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
        use console_error_panic_hook::set_once as set_panic_hook;
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

#[derive(Debug, Clone)]
struct Segment {
  x0: u32,
  x1: u32,
  y0: u32,
  y1: u32,
}

#[wasm_bindgen]
pub struct RandomImage {
  width: u32,
  height: u32,
  pixels: Vec<Pixel>,
  segments: Vec<Segment>,
}

#[wasm_bindgen]
impl RandomImage {
  pub fn new(width: u32, height: u32) -> RandomImage {
    let size = (width * height) as usize;
    let pixels: Vec<Pixel> = (0..size).map(|_| Pixel::new()).collect();

    let segment_count = 8;
    let mut segments = vec![];
    let mut rng = OsRng::new().unwrap();
    for _ in 0..segment_count {
      let x0 = rng.gen_range(0, width);
      let x1 = rng.gen_range(0, width);
      let y0 = rng.gen_range(0, height);
      let y1 = rng.gen_range(0, height);
      segments.push(Segment { x0, x1, y0, y1 })
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

  fn _render_vert_line(&self, pixels: &mut [u8], x0: i32, y0: i32, y1: i32) {
    let x = x0;
    for y in y0..y1 {
      if self.in_bounds(x, y) {
        let index = self.pixel_sub_index(x as u32, y as u32);
        pixels[index] = 0;
        pixels[index + 1] = 0;
        pixels[index + 2] = 0;
        pixels[index + 3] = 255;
      }
    }
  }

  fn _render_horiz_line(&self, pixels: &mut [u8], x0: i32, y0: i32, x1: i32) {
    let y = y0;
    for x in x0..x1 {
      if self.in_bounds(x, y) {
        let index = self.pixel_sub_index(x as u32, y as u32);
        pixels[index] = 0;
        pixels[index + 1] = 0;
        pixels[index + 2] = 0;
        pixels[index + 3] = 255;
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

  // the first index (of 4) for this pixel
  fn pixel_sub_index(&self, x: u32, y: u32) -> usize {
    4 * self.pixel_index(x, y)
  }

  pub fn render(&self, pixels: &mut [u8]) {
    for segment in &self.segments {
      self.render_line(
        pixels,
        segment.x0 as i32,
        segment.y0 as i32,
        segment.x1 as i32,
        segment.y1 as i32,
      );
    }
  }

  pub fn render_line(&self, pixels: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32) {
    if y0 == y1 {
      return self._render_horiz_line(pixels, y0, x0, x1);
    } else if x0 == x1 {
      return self._render_vert_line(pixels, x0, y0, y1);
    }
    if x0 > x1 {
      // re-order so that we go from left to right
      return self.render_line(pixels, x1, y1, x0, y0);
    }
    let delta_x = x1 - x0;
    let delta_y = y1 - y0;
    let delta_err: f32 = (delta_y as f32 / delta_x as f32).abs();
    let mut error: f32 = 0.0;
    let mut y = y0;
    for x in x0..x1 {
      if self.in_bounds(x, y) {
        let index = self.pixel_sub_index(x as u32, y as u32);
        pixels[index] = 0;
        pixels[index + 1] = 0;
        pixels[index + 2] = 0;
        pixels[index + 3] = 255;
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
        shrunk_img.set_pixel(tile_col, tile_row, avg_pixel);
      }
    }

    shrunk_img
  }

  /*
  An idea for an alternative approach to shrinking the image by iterating only once through the
  pixels of the image and determining at each one what its (x,y) coord in the shrunk image would be,
  then using a HashMap to keep track of the sums of r,g,b in each output pixel,
  and finally building up the shrunk pixel Vec.
  This function is about an order of magnitude slower than the other one when run in WASM, ~700ms.
  */
  pub fn shrink_via_hashmap(&self, width: u32, height: u32) -> RandomImage {
    let tile_width = self.width / width;
    let tile_height = self.height / height;

    let size: usize = (tile_width * tile_height) as usize;

    let mut buckets: HashMap<(u32, u32), (u32, u32, u32)> = HashMap::new();

    let mut pixel_idx = 0;
    for pixel in &self.pixels {
      let image_y = pixel_idx / self.height;
      let image_x = pixel_idx - image_y * self.width;

      // Corresponding output pixel that this input pixel will shrink down to
      let shrink_x = image_x / tile_width;
      let shrink_y = image_y / tile_height;

      let (r, g, b) = buckets.entry((shrink_x, shrink_y)).or_insert((0, 0, 0));
      *r += pixel.r as u32;
      *g += pixel.g as u32;
      *b += pixel.b as u32;
      pixel_idx += 1;
    }

    let mut pixels: Vec<Pixel> = Vec::with_capacity(size);
    for shrink_y in 0..(self.height / tile_height) {
      for shrink_x in 0..(self.width / tile_width) {
        let (r_sum, g_sum, b_sum) = match buckets.get(&(shrink_x, shrink_y)) {
          Some(v) => v,
          None => {
            log!(
              "Failed to find value for bucket at ({},{})",
              shrink_x,
              shrink_y
            );
            panic!("argh");
          }
        };
        pixels.push(Pixel {
          r: (*r_sum / size as u32) as u8,
          g: (*g_sum / size as u32) as u8,
          b: (*b_sum / size as u32) as u8,
          a: 255,
        });
      }
    }

    let shrunk_img = RandomImage {
      width,
      height,
      pixels,
      segments: vec![],
    };
    shrunk_img
  }

  // TODO - this doesn't actually put it in the right place, it just pushes it
  // onto the pixels Vec. It works because it is called in the correct row-by-row-then-col-by-col order
  pub fn set_pixel(&mut self, x: u32, y: u32, p: Pixel) {
    // let idx = self.pixel_index(x, y);
    self.pixels.push(p);
    // self.pixels[idx] = p;
  }

  // TODO - Is this a speed/memory issue, that the pixel must be cloned? It is a lightweight struct so
  // I would expect it to be fast. wasm-bindgen will not compile if this returns a ref to a pixel.
  pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
    self.pixels[self.pixel_index(x, y)].clone()
  }

  pub fn drop(&self) {
    std::mem::drop(self);
  }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
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

  pub fn add_color(&mut self) {
    self.r = 0;
    self.g = 0;
    self.b = 0;
  }
}
