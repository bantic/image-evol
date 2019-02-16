#[macro_use]
extern crate cfg_if;
extern crate rand;
extern crate wasm_bindgen;

use rand::rngs::OsRng;
use rand::Rng;

use wasm_bindgen::prelude::*;

extern crate web_sys;

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

  pub fn shrink(&self, width: u32, height: u32) -> RandomImage {
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
            let pixel = self.get_pixel(x, y);
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

  pub fn set_pixel(&mut self, x: u32, y: u32, p: Pixel) {
    let idx = self.pixel_index(x, y);
    self.pixels.push(p);
    // self.pixels[idx] = p;
  }

  pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
    self.pixels[self.pixel_index(x, y)].clone()
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
