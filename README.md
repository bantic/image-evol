# image-evol

An experiment in writing a genetic evolution algorithm that runs in wasm in the browser.

# todos

- [x] write the similarity code
- [x] write mutation code
- [x] write loop to mutate
- [x] try to make mutate faster -- mutate in-place (this worked -- way, way faster)
- [x] try to make mutate faster -- see how it changes with _many_ more line segments -- no problem with even as many as 500 segments
- [ ] create a pool of mutation candidates and select the best ones at each iteration -> improve the convergence

## later todos

- [ ] add filled triangles
- [ ] colors & transparency
- [ ] sexual selection, in addition to mutation
- [ ] load an image
- [ ] UI controls to modify mutation rates

- `npm run start` -- Serve the project locally for development at
  `http://localhost:8080`.

- `npm run build` -- Bundle the project (in production mode).
