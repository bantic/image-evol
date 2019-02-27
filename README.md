# image-evol

An experiment in writing a genetic evolution algorithm that runs in wasm in the browser.

# todos

- [x] write the similarity code
- [x] write mutation code
- [x] write loop to mutate
- [ ] try to make mutate faster -- mutate in-place
- [ ] try to make mutate faster -- see how it changes with _many_ more line segments

## later todos

- [ ] add circles (ellipses, affine transforms)
- [ ] colors & transparency
- [ ] sexual selection, in addition to mutation
- [ ] load an image
- [ ] UI controls to modify mutation rates

- `npm run start` -- Serve the project locally for development at
  `http://localhost:8080`.

- `npm run build` -- Bundle the project (in production mode).
