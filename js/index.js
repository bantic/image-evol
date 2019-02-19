import('../crate/pkg').then(module => {
  run(module);
});

function run(wasm) {
  let els = {
    canvas: document.getElementById('canvas'),
    canvasShrink: document.getElementById('canvas-shrink'),
    buttonJS: document.getElementById('button-js'),
    buttonWASM: document.getElementById('button-wasm')
  };

  let width = 500,
    height = 500;

  let image = wasm.RandomImage.new(width, height);

  drawImageFromWASMMemory(els.canvas, image, wasm);

  // WASM-based shrinking
  els.buttonWASM.addEventListener('click', () => {
    console.time('shrinkWASM');
    let newImage = image.shrink(100, 100);
    console.timeEnd('shrinkWASM');
    drawImageFromWASMMemory(els.canvasShrink, newImage, wasm);
    newImage.drop(); // free memory on WASM side
  });
  els.buttonJS.addEventListener('click', () => {
    console.time('shrinkJS');
    let pixelData = shrinkImageJS(image, wasm, 100, 100);
    console.timeEnd('shrinkJS');
    drawImageFromPixelData(els.canvasShrink, pixelData);
  });
}

// Find the slice of WASM memory that corresponds to the image's
// pixels, and `putImage` those into the given canvas
function drawImageFromWASMMemory(canvas, image, wasm) {
  let width = image.width(),
    height = image.height();
  canvas.width = width;
  canvas.height = height;

  console.time('drawImageWithWASMMemory');
  let mem = wasm.get_memory();
  let pixels = image.pixels();
  let data = new Uint8ClampedArray(mem.buffer, pixels, 4 * image.size());
  let imageData = new ImageData(data, width, height);

  let ctx = canvas.getContext('2d');
  ctx.putImageData(imageData, 0, 0);
  console.timeEnd('drawImageWithWASMMemory');
}

// pixelData is {width,height, data: Uint8ClampedArray }
// Creates image data from pixel data array and
// putImageData's to canvas
function drawImageFromPixelData(canvas, pixelData) {
  let { height, width, data } = pixelData;
  canvas.width = width;
  canvas.height = height;

  console.time('drawImageFromPixelData');
  let imageData = new ImageData(data, width, height);

  let ctx = canvas.getContext('2d');
  ctx.putImageData(imageData, 0, 0);
  console.timeEnd('drawImageFromPixelData');
}

// This is an experiment to try giving a slice of
// data from JS to WASM and having the WASM code modify
// those pixels directly. It is about 33% slower than
// drawImageFromWASMMemory
function drawImageToExistingPixels(canvas, image) {
  let width = image.width(),
    height = image.height();
  canvas.width = width;
  canvas.height = height;

  console.time('drawImageToExistingPixels');
  let data = new Uint8Array(width * height * 4);
  image.render(data);
  let imageData = new ImageData(new Uint8ClampedArray(data), width, height);

  let ctx = canvas.getContext('2d');
  ctx.putImageData(imageData, 0, 0);
  console.timeEnd('drawImageToExistingPixels');
}

// Shrink the image wholly in JS
function shrinkImageJS(image, wasm, shrink_width, shrink_height) {
  let mem = wasm.get_memory();
  let width = image.width(),
    height = image.height();
  let pixels = image.pixels();
  let data = new Uint8Array(mem.buffer, pixels, 4 * image.size());

  let new_data = new Uint8ClampedArray(shrink_width * shrink_height * 4);
  let tile_width = Math.floor(width / shrink_width);
  let tile_height = Math.floor(height / shrink_height);
  let tile_size = tile_width * tile_height;

  for (let tile_row = 0; tile_row < shrink_height; tile_row++) {
    for (let tile_col = 0; tile_col < shrink_width; tile_col++) {
      let sums = [0, 0, 0]; // r, g, b
      for (
        let x = tile_col * tile_width;
        x < (tile_col + 1) * tile_width;
        x++
      ) {
        for (
          let y = tile_row * tile_height;
          y < (tile_row + 1) * tile_height;
          y++
        ) {
          let idx = 4 * (y * width + x);
          sums[0] += data[idx]; // r
          sums[1] += data[idx + 1]; // g
          sums[2] += data[idx + 2]; // b
        }
      }

      let avgs = sums.map(color => color / tile_size);
      let shrink_idx = 4 * (tile_row * shrink_width + tile_col);
      new_data[shrink_idx] = avgs[0];
      new_data[shrink_idx + 1] = avgs[1];
      new_data[shrink_idx + 2] = avgs[2];
      new_data[shrink_idx + 3] = 255; // alpha
    }
  }

  return { width: shrink_width, height: shrink_height, data: new_data };
}
