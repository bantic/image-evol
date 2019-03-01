# image-evol

An experiment in writing a genetic evolution algorithm that runs in wasm in the browser.

![gif of image evoluation](image-evol.gif)

Curious? There is [a presentation](https://docs.google.com/presentation/d/1qxn7Ype4TxMTzMqR1i-4HhhbhUwFkZeqJ7f6e_lw9eA/edit#slide=id.p) covering my learnings while working on this, and a [deployed site](http://rust-wasm-image-evolution.surge.sh/) that hosts the various demos that show different points of progress.

# todos

- [x] write the similarity code
- [x] write mutation code
- [x] write loop to mutate
- [x] try to make mutate faster -- mutate in-place (this worked -- way, way faster)
- [x] try to make mutate faster -- see how it changes with _many_ more line segments -- no problem with even as many as 500 segments

- [x] add filled triangles
- [x] replace line segments with triangles
- [x] colors & transparency
- [ ] implement population selection - mutation, tournament selection
- [ ] create a pool of mutation candidates and select the best ones at each iteration -> improve the convergence

## later todos

- [ ] remove unneeded nalgebra crate?
- [x] load an image as a reference rather than random
- [ ] UI controls to modify mutation rates

- `npm run start` -- Serve the project locally for development at
  `http://localhost:8080`.

- `npm run build` -- Bundle the project (in production mode).
