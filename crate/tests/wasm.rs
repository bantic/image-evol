extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

extern crate rust_webpack;
use rust_webpack::{Image, Pixel};

#[wasm_bindgen_test]
fn test_image() {
  println!("HELLO");
  let mut i = Image::new(1, 2);
  assert_eq!(i.size(), 2);

  let p = i.get_pixel(0, 0);
  assert_eq!(p.r, 0);

  i.invert();
  let p = i.get_pixel(0, 0);
  assert_eq!(p.r, 255);
}
