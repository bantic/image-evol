import('../crate/pkg').then(module => {
  run(module);
});

function run(wasm) {
  let els = {
    canvas: document.getElementById('canvas'),
    canvasShrink: document.getElementById('canvas-shrink'),
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
