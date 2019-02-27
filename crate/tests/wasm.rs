extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;

extern crate image_evol;
use image_evol::RandomImage;

#[wasm_bindgen_test]
fn test_image() {
  let i = RandomImage::new(1, 2);
  assert_eq!(i.size(), 2);

  let p = i.get_pixel(0, 0);
  assert_eq!(p.r, 0);
}
