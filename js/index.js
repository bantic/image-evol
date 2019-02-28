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

  drawImageFromWASMMemory(els.canvas, referenceImage, wasm);

  let { width: widthShrunk, height: heightShrunk } = SHRUNK_DIMS;
  let shrunkReferenceImage = referenceImage.shrink(widthShrunk, heightShrunk);
  let err = Infinity;

  let population = [];
  for (let i = 0; i < POP_SIZE; i++) {
    population.push({
      image: wasm.RandomImage.new(width, height),
      err: Infinity
    });
  }

  els.buttonWASM.addEventListener('click', () => {
    let mem = wasm.get_memory();
    let data = new Uint8ClampedArray(
      mem.buffer,
      shrunkReferenceImage.pixels(),
      4 * shrunkReferenceImage.size()
    );
    let pop = wasm.Population.new(10, 10, data);
    pop.add_member();
    console.log(pop.best_fitness());
    // requestAnimationFrame(update);
  });

  let iterations = 0;
  let MAX_ITER = Infinity;

  function update() {
    iterations++;
    if (iterations > MAX_ITER) {
      return;
    }

    for (let item of population) {
      if (item.err == err && err !== Infinity) {
        continue;
      }
      item.image.mutate();
      let newErr = item.image.calculate_fitness(shrunkReferenceImage);
      item.err = newErr;
    }

    let sorted = population.sort((a, b) => {
      return a.err < b.err ? -1 : a.err > b.err ? 1 : 0;
    });
    let best = sorted[0];
    // reset bottom 2
    // TODO bring back population culling
    // sorted[sorted.length - 1].image.reset();
    // sorted[sorted.length - 2].image.reset();

    console.log(`err ${err} -> ${best.err}`);
    err = best.err;
    drawImageFromWASMMemory(els.canvas2, best.image, wasm);
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
