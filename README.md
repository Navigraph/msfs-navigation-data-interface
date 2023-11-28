# Navigraph Navdata Interface in MSFS

This is a barebones implementation to be able to download up-to-date Navigraph navdata into the sim (more specifically into the `work` folder of the aircraft).

Documentation on the events used on the CommBus is located [here](/DOCS.md)

## Repository Structure

Here's an overview on the structure of this repository, which is designed to be as simple as possible to use

- `examples/`
  - Contains sample implementations for using the navdata interface
  - `aircraft/` includes a base aircraft to test in the sim
  - `gauge/` includes a very simple TypeScript instrument to communicate with the WASM module
- `src/`
  - Contains the source for the navdata interface (and soon the JS library)
  - `wasm_navdata_interface` includes the Rust source code for the WASM module

## Including in Your Aircraft

1. You'll need to either build the WASM module yourself (not recommended, but documented further down) or download it from [the latest release](https://github.com/Navigraph/msfs-navdata-interface/releases).
2. Add the WASM module into your `panel` folder in `PackageSources`
3. Add the following entry into `panel.cfg` (make sure to replace `NN` with the proper `VCockpit` ID):
   ```
   [VCockpitNN]
   size_mm=0,0
   pixel_size=0,0
   texture=NO_TEXTURE
   htmlgauge00=WasmInstrument/WasmInstrument.html?wasm_module=navdata_interface.wasm&wasm_gauge=navdata_interface,0,0,1,1
   ```
   - Note that if you already have a `VCockpit` with `NO_TEXTURE` you can just add another `htmlgauge` to it, while making sure to increase the index

## Building the Sample Aircraft

Before building, make sure you have properly created and set an `.env` file in `src/gauge`! An example can be found in the `.env.example` file in that directory. Replace with your credentials

1. [Download](https://nodejs.org/en/download) Node.js
2. Open the `src/gauge` folder in a terminal
3. Run `npm i` the first time you build, in order to install dependencies
4. Run `npm run build` to build into the `PackageSources` folder of the aircraft sample (or `npm run dev` to build into the `Packages` folder of the aircraft and listen to changes in the source).
5. Make sure the WASM module is included in the `panel` folder! Look at either [Including in Your Aircraft](#including-in-your-aircraft) or [Building the WASM Module Yourself](#building-the-wasm-module-yourself) for info on that
6. Open the `examples/aircraft/NavdataInterfaceAircraftProject.xml` file in the simulator and build there

## Building the WASM Module Yourself

1. [Download](https://www.docker.com/products/docker-desktop/) Docker Desktop
2. Open the `src/wasm_navdata_interface` folder in a terminal
3. Run `.\build.bat` (must be on Windows)
   - This will take a while to download and build the first time, but subsequent runs will be quicker
4. The compiled WASM module will be copied to `src/wasm_navdata_interface/out` **and** `examples/aircraft/PackageSources/SimObjects/Airplanes/Navigraph_Navdata_Interface_Aircraft/panel`
