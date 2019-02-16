import('../crate/pkg').then(module => {
  run(module);
});

function run(wasm) {
  let els = {
    canvas: document.getElementById('canvas'),
    canvas2: document.getElementById('canvas2'),
    button: document.getElementById('button')
  };

  let width = 500,
    height = 500;

  let image = wasm.RandomImage.new(width, height);

  let draw = () => drawImage(els.canvas, image, wasm);
  draw();

  els.button.addEventListener('click', () => {
    let now = new Date();
    let newImage = image.shrink(100, 100);
    console.log('shrink in: ', new Date() - now);
    drawImage(els.canvas2, newImage, wasm);
  });
}

function drawImage(canvas, image, wasm) {
  let width = image.width(),
    height = image.height();
  canvas.width = width;
  canvas.height = height;
  let mem = wasm.get_memory();
  let pixels = image.pixels();
  let data = new Uint8ClampedArray(mem.buffer, pixels, 4 * image.size());
  let imageData = new ImageData(data, width, height);

  let ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, width, height);
  ctx.putImageData(imageData, 0, 0);
}
