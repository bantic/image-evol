import('../crate/pkg').then(module => {
  run(module);
});

function run(wasm) {
  let els = {
    canvas: document.getElementById('canvas'),
    button: document.getElementById('button')
  };

  let width = 300,
    height = 300;
  els.canvas.width = width;
  els.canvas.height = height;

  let image = wasm.RandomImage.new(width, height);

  let draw = () => drawImage(els.canvas, image, wasm);
  draw();

  els.button.addEventListener('click', () => {
    image.line(50, 25, 150, 25);
    image.line(50, 25, 50, 100);
    draw();
  });
}

function drawImage(canvas, image, wasm) {
  let width = image.width(),
    height = image.height();
  let mem = wasm.get_memory();
  let pixels = image.pixels();
  let data = new Uint8ClampedArray(mem.buffer, pixels, 4 * image.size());
  let imageData = new ImageData(data, width, height);

  let ctx = canvas.getContext('2d');
  ctx.clearRect(0, 0, width, height);
  ctx.putImageData(imageData, 0, 0);
}
