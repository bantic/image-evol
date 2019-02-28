import('../crate/pkg').then(module => {
  run(module);
});

const LARGE_IMAGE_DIMS = { width: 300, height: 300 };
const SHRUNK_DIMS = { width: 10, height: 10 };
const POP_SIZE = 10;

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
  let referenceImage = wasm.RandomImage.new(width, height);
  console.timeEnd('RandomImage.new');
  console.time('RandomImage.render');
  referenceImage.render();
  console.timeEnd('RandomImage.render');

  drawImageFromWASMMemory(els.canvas, referenceImage, wasm);

  let { width: widthShrunk, height: heightShrunk } = SHRUNK_DIMS;
  let shrunkReferenceImage = referenceImage.shrink(widthShrunk, heightShrunk);
  let err = Infinity;

  let pop = wasm.Population.new(
    width,
    height,
    shrunkReferenceImage.pixels(),
    SHRUNK_DIMS.width,
    SHRUNK_DIMS.height
  );
  for (let i = 0; i < POP_SIZE; i++) {
    pop.add_member();
  }

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
    let best = pop.best_fitness();
    drawPixels(els.canvas2, pop.best_pixels(), width, height, wasm);
    pop.evolve();

    // console.log(`err ${err} -> ${best.err}`);
    // err = best.err;
    // drawImageFromWASMMemory(els.canvas2, best.image, wasm);
    updateUI(els, iterations, best);
    requestAnimationFrame(update);
  }
}

function updateUI({ uiIterations, uiError }, iterations, err) {
  uiIterations.textContent = `Iterations: ${iterations}`;
  uiError.textContent = `Err: ${err}`;
}

function drawPixels(canvas, pixelsPtr, width, height, wasm) {
  canvas.width = width;
  canvas.height = height;
  let mem = wasm.get_memory();
  let data = new Uint8ClampedArray(mem.buffer, pixelsPtr, 4 * width * height);
  let imageData = new ImageData(data, width, height);
  let ctx = canvas.getContext('2d');
  ctx.putImageData(imageData, 0, 0);
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
  let pixelsPtr = image.pixels();
  let data = new Uint8ClampedArray(mem.buffer, pixelsPtr, 4 * image.size());
  let imageData = new ImageData(data, width, height);

  let ctx = canvas.getContext('2d');
  ctx.putImageData(imageData, 0, 0);
  // console.timeEnd('drawImageWithWASMMemory');
}
