import('../crate/pkg').then(module => {
  run(module);
});

const LARGE_IMAGE_DIMS = { width: 500, height: 500 };
const SHRUNK_DIMS = { width: 10, height: 10 };

function run(wasm) {
  let els = {
    canvas: document.getElementById('canvas'),
    canvas2: document.getElementById('canvas2'),
    canvasShrink: document.getElementById('canvas-shrink'),
    buttonWASM: document.getElementById('button-wasm'),
    uiIterations: document.getElementById('ui-iterations'),
    uiError: document.getElementById('ui-error')
  };

  let { width, height } = LARGE_IMAGE_DIMS;

  console.time('RandomImage.new');
  let image = wasm.RandomImage.new(width, height);
  console.timeEnd('RandomImage.new');
  console.time('RandomImage.new');
  let mutatedImage = wasm.RandomImage.new(width, height);
  console.timeEnd('RandomImage.new');

  drawImageFromWASMMemory(els.canvas, image, wasm);

  let { width: widthShrunk, height: heightShrunk } = SHRUNK_DIMS;
  let referenceImage = image.shrink(widthShrunk, heightShrunk);
  let err = Infinity;

  // WASM-based shrinking
  els.buttonWASM.addEventListener('click', () => {
    requestAnimationFrame(update);
  });

  let iterations = 0;
  let MAX_ITER = Infinity;

  function update() {
    iterations++;
    if (iterations > MAX_ITER) {
      return;
    }
    console.time('mutate');
    mutatedImage.mutate();
    console.timeEnd('mutate');
    console.time('shrink');
    let shrunk = mutatedImage.shrink(widthShrunk, heightShrunk);
    console.timeEnd('shrink');
    console.time('compareError');
    let newErr = referenceImage.compare(shrunk);
    console.timeEnd('compareError');
    if (newErr < err) {
      err = newErr;
      drawImageFromWASMMemory(els.canvas2, mutatedImage, wasm);
    }
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
