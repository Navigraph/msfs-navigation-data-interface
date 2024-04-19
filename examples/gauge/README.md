# Gauge example

This is a simple JavaScript gauge to interface with the WASM module.

## Building

To build to your `Packages` folder and listen to changes (i.e. when you're debugging or quickly prototyping) you can run `npm run dev`. Otherwise, run `npm run build`

- Note: `npm run dev` will output once into the `PackageSources` folder initially to prevent cases where the `build` command isn't ran before building in the sim which would lead to errors
