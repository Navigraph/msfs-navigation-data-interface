# Navigraph Navdata Interface in MSFS

This is a barebones implementation to be able to download up-to-date Navigraph navdata into MSFS (more specifically into the `work` folder of the aircraft).

Documentation on the events used on the CommBus is located [here](/DOCS.md)

## Repository Structure

Here's an overview on the structure of this repository, which is designed to be as simple as possible to use

- `examples/`
  - Contains sample implementations for using the navdata interface
  - `aircraft/` includes a base aircraft to test in the sim
  - `gauge/` includes a very simple TypeScript instrument to communicate with the WASM module
- `src/`
  - `js` Includes source code for the JS interface for using the sdk
  - `test` Includes code for testing the JS and Rust code using a Node runtime
  - `wasm` includes the Rust source code for the WASM module which handles the database interface

## Including in Your Aircraft

1. You'll need to either build the WASM module yourself (not recommended, but documented further down) or download it from [the latest release](https://github.com/Navigraph/msfs-navdata-interface/releases) (alternatively you can download it off of a commit by looking at the uploaded artifacts).
2. Add the WASM module into your `panel` folder in `PackageSources`
3. Add the following entry into `panel.cfg` (make sure to replace `NN` with the proper `VCockpit` ID):
   ```
   [VCockpitNN]
   size_mm=0,0
   pixel_size=0,0
   texture=NO_TEXTURE
   htmlgauge00=WasmInstrument/WasmInstrument.html?wasm_module=msfs_navdata_interface.wasm&wasm_gauge=navdata_interface,0,0,1,1
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
2. Run `npm run build:wasm` (must be on Windows)
   - This will take a while to download and build the first time, but subsequent runs will be quicker
3. The compiled WASM module will be copied to `out` **and** `examples/aircraft/PackageSources/SimObjects/Airplanes/Navigraph_Navdata_Interface_Aircraft/panel`

## Interfacing with the navdata gauge manually

The navdata interface acts as its own WASM gauge in sim, so in order to communicate with it, you must use the [CommBus](https://docs.flightsimulator.com/html/Programming_Tools/WASM/Communication_API/Communication_API.htm).

The gauge communicates using the following event names:

(Any types referenced can be found in `wasm/src/json_structs.rs`)

- `NAVIGRAPH_CallFunction`: This event is received by the interface and is used to trigger one of the interfaces functions. It takes in arguments of type `CallFunction`. The available functions and their expected parameters can be found in the `json_structs.rs` file
- `NAVIGRAPH_FunctionResult`: This event is sent by the interface as a response to a previously triggered function. Its result will have the type `FunctionResult`, with the data field containing the expected return type of the function.
- `NAVIGRAPH_Event`: This event is sent by the interface to give indications of progress or that the interface is running correctly.

### Example

Below is an example of communicating with the interface in JS. (We provide a JS wrapper, the code below is just a basic example to show how it works). Please read the CommBus documentation to determine how to interface with CommBus in your chosen language. `src/js` contains our JS wrapper, it is also a useful example for implementing a fully fleshed out wrapper.

```js
const queue[] = []

const listener = RegisterCommBusListener(() => {
  listener.on('NAVIGRAPH_FunctionResult', (jsonArgs) => {
    const args = JSON.parse(jsonArgs)

    // When a FunctionResult is received, find the item in queue which matches the id, and resolve or reject it
    const queueItem = queue.find(m => m.id === args.id)

    if(queueItem) {
      queue.splice(queue.indexOf(queueItem), 1)
        const data = args.data

        if (args.status === FunctionResultStatus.Success) {
          queueItem.resolve(data)
        } else {
          queueItem.reject(new Error(typeof data === "string" ? data : "Unknown error"))
        }
      }
  })
}) // RegisterCommBusListener is a function provided by sim

function getAirport(ident) {
  const id = Utils.generateGUID() // Utils is a class provided by sim

  const args = {
    function: "GetAirport", // The name of the function being called
    id, // CallFunctions and FunctionResults are tied together with the id field
    data: { // The parameters of the function
      ident
    },
  }

  listener.callWasm("NAVIGRAPH_CallFunction", JSON.stringify(args))

  return new Promise((resolve, reject) => {
    queue.push({
      id,
      resolve: (response) => resolve(response),
      reject: (error) => reject(error),
    })
  })
}

function executeSql(sql, params) {
  const id = Utils.generateGUID() // Utils is a class provided by sim

  const args = {
    function: "ExecuteSQLQuery", // The name of the function being called
    id, // CallFunctions and FunctionResults are tied together with the id field
    data: { // The parameters of the function
      sql,
      params
    },
  }

  listener.callWasm("NAVIGRAPH_CallFunction", JSON.stringify(args))

  return new Promise((resolve, reject) => {
    queue.push({
      id,
      resolve: (response) => resolve(response),
      reject: (error) => reject(error),
    })
  })
}
```
