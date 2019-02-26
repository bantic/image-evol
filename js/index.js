import('../crate/pkg').then(module => {
  run(module);
});

function run(wasm) {
  let els = {
    canvas: document.getElementById('canvas'),
    canvas2: document.getElementById('canvas2'),
    canvasShrink: document.getElementById('canvas-shrink'),
    buttonWASM: document.getElementById('button-wasm')
  };

  let width = 500,
    height = 500;

  let image = wasm.RandomImage.new(width, height);

  drawImageFromWASMMemory(els.canvas, image, wasm);

  let referenceImage = image.shrink(10, 10);
  let diff = Infinity;

  // WASM-based shrinking
  els.buttonWASM.addEventListener('click', () => {
    requestAnimationFrame(update);
  });

  function update() {
    let newImage = wasm.RandomImage.new(width, height);
    let newDiff = referenceImage.compare(newImage.shrink(10, 10));
    if (newDiff < diff) {
      console.log(`Diff ${diff} -> ${newDiff} (${diff - newDiff})`);
      diff = newDiff;
      drawImageFromWASMMemory(els.canvas2, newImage, wasm);
    }
    requestAnimationFrame(update);
  }
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
