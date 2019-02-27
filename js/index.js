import('../crate/pkg').then(module => {
  run(module);
});

function run(wasm) {
  let els = {
    canvas: document.getElementById('canvas'),
    canvas2: document.getElementById('canvas2'),
    canvasShrink: document.getElementById('canvas-shrink'),
    buttonWASM: document.getElementById('button-wasm'),
    uiIterations: document.getElementById('ui-iterations'),
    uiError: document.getElementById('ui-error')
  };

  let width = 500,
    height = 500;

  let image = wasm.RandomImage.new(width, height);
  let mutatedImage = wasm.RandomImage.new(width, height);

  drawImageFromWASMMemory(els.canvas, image, wasm);

  let referenceImage = image.shrink(10, 10);
  let err = Infinity;

  // WASM-based shrinking
  els.buttonWASM.addEventListener('click', () => {
    requestAnimationFrame(update);
  });

  let iterations = 0;

  function update() {
    iterations++;
    if (iterations > 10) {
      return;
    }
    console.time('mutate');
    let newImage = mutatedImage.mutate();
    console.timeEnd('mutate');
    console.time('shrink');
    let shrunk = newImage.shrink(10, 10);
    console.timeEnd('shrink');
    console.time('compareError');
    let newErr = referenceImage.compare(shrunk);
    console.timeEnd('compareError');
    if (newErr < err) {
      err = newErr;
      mutatedImage = newImage;
    }
    drawImageFromWASMMemory(els.canvas2, mutatedImage, wasm);
    updateUI(els, iterations, err);
    requestAnimationFrame(update);
  }
}

function updateUI({ uiIterations, uiError }, iterations, err) {
  uiIterations.textContent = `Iterations: ${iterations}`;
  uiError.textContent = `Err: ${err}`;
}

// Find the slice of WASM memory that corresponds to the image's
// pixels, and `putImage` those into the given canvas
function drawImageFromWASMMemory(canvas, image, wasm) {
  let width = image.width(),
    height = image.height();
  canvas.width = width;
  canvas.height = height;

  // console.time('drawImageWithWASMMemory');
  let mem = wasm.get_memory();
  let pixels = image.pixels();
  let data = new Uint8ClampedArray(mem.buffer, pixels, 4 * image.size());
  let imageData = new ImageData(data, width, height);

  let ctx = canvas.getContext('2d');
  ctx.putImageData(imageData, 0, 0);
  // console.timeEnd('drawImageWithWASMMemory');
}
