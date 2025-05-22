<div align="center" >
  <a href="https://navigraph.com">
    <img src="https://navigraph.com/assets/images/navigraph_logo_only.svg" alt="Logo" width="80" height="80">
  </a>

  <div align="center">
    <h1>Navigraph Navigation Data Interface for MSFS</h1>
  </div>

  <p>The Navigraph Navigation Data Interface enables developers to download and integrate navigation data from Navigraph directly into add-on aircraft in MSFS.</p>

  <br/>
</div>

## Key Features

- Navigraph DFD Format: Leverage specialized support for Navigraph's DFD format, based on SQLite, which includes an SQL interface on the commbus for efficient data handling.
- Javascript and WASM support: The navdata interface is accessible from both Javascript (Coherent) and WASM, providing flexibility for developers.
- Xbox compatibility: Works on PC and Xbox.
- Persistence: All data is persisted in the `work` folder of the aircraft.

## Repository Structure

- `example/`
  - `aircraft/` includes a base aircraft to test in the sim
  - `gauge/` includes a very simple TypeScript instrument to communicate with the WASM module
- `src/`
  - `ts` includes source code for the JS interface for interfacing with the WASM module
  - `wasm` includes the Rust source code for the WASM module which handles the downloading of the database file, and interfacing with the database

## Including in Your Aircraft

1. You'll need to either build the WASM module yourself (not recommended, but documented further down) or download it from [the latest release](https://github.com/Navigraph/msfs-navigation-data-interface/releases) (alternatively you can download it off of a commit by looking at the uploaded artifacts).
2. Add the WASM module into your `panel` folder in `PackageSources`
3. Add the following entry into `panel.cfg` (make sure to replace `NN` with the proper `VCockpit` ID):

   ```ini
   [VCockpitNN]
   size_mm=0,0
   pixel_size=0,0
   texture=NO_TEXTURE
   htmlgauge00=WasmInstrument/WasmInstrument.html?wasm_module=msfs_navigation_data_interface.wasm&wasm_gauge=navigation_data_interface,0,0,1,1
   ```

   - Note that if you already have a `VCockpit` with `NO_TEXTURE` you can just add another `htmlgauge` to it, while making sure to increase the index

4. **Optional**: Create a `Navigraph/config.json` file to provide additional metadata at runtime. This info will be reported to us should any error occur in the library, enabling us to directly reach out to you (the developer) to help track down the issue.

   - The file must follow this format:

   ```json
   {
     "addon": {
       "developer": "Navigraph",
       "product": "Sample Aircraft"
     }
   }
   ```

## Dealing with Bundled Navigation Data

If you bundle outdated navigation data in your aircraft and you want this module to handle updating it for users with subscriptions, place the navigation data into the `Navigraph/BundledData` directory in `PackageSources`. You can see an example [here](example/aircraft/PackageSources/Navigraph/BundledData/)

The navigation data interface will automatically use this database by default, making it immediately available on startup.

## Where is the Navigation Data Stored?

The default location for navigation data is `work/NavigationData`.

## Building the Sample Aircraft (MSFS2020)

Before building, make sure you have properly created and set an `.env` file in `example/gauge`! An example can be found in the `.env.example` file in that directory. Replace with your credentials

1. Download and install [Bun](https://bun.sh/docs/installation)
2. Open the `msfs-navigation-data-interface` folder in a terminal
3. Run `bun i` the first time you build, in order to install dependencies
4. Change directory to `example/gauge` using `cd example/gauge`
5. Run `bun run build` to build into the `PackageSources` folder of the aircraft sample (or `bun run dev` to build into the `Packages` folder of the aircraft and listen to changes in the source).
6. Make sure the WASM module is included in the [`panel`](example/aircraft/PackageSources/SimObjects/Airplanes/Navigraph_Navigation_Data_Interface_Aircraft/panel) folder! Look at either [Including in Your Aircraft](#including-in-your-aircraft) or [Building the WASM Module Yourself](#building-the-wasm-module-yourself) for info on that
7. Open the `example/aircraft/NavigationDataInterfaceAircraftProject.xml` file in the simulator and build there

## Building the WASM Module Yourself

1. Create a `.env` file in the root of this repository, containing a `SENTRY_URL` variable. Provide your own DSN, or leave it empty.
2. Run `bun run build:wasm` at the root of the repository (requires Docker)
   - This will take a while to download and build the first time, but subsequent runs will be quicker
3. The compiled WASM module will be copied to `dist/wasm`. There will be two folders - `2020` and `2024`, for each sim version

## Interfacing with the gauge manually

The navigation data interface acts as its own WASM gauge in sim, so in order to communicate with it, you must use the [CommBus](https://docs.flightsimulator.com/html/Programming_Tools/WASM/Communication_API/Communication_API.htm).

The gauge communicates using the following event names (all types referenced can be found [here](src/ts)):

- `NAVIGRAPH_CallFunction`: This event is received by the interface and is used to trigger one of the interfaces functions. It takes in arguments of type `CallFunction`. The available functions and their expected parameters can be found in the [`src/ts`](src/ts) file
- `NAVIGRAPH_FunctionResult`: This event is sent by the interface as a response to a previously triggered function. Its result will have the type `FunctionResult`, with the data field containing the expected return type of the function.
- `NAVIGRAPH_Event`: This event is sent by the interface to give indications of progress or that the interface is running correctly.

### Example

Below is an example of communicating with the interface in JS. Please read the CommBus documentation to determine how to interface with CommBus in your chosen language. [`src/ts`](src/ts) contains our JS wrapper, it is also a useful example for implementing a fully fleshed out wrapper.

> [!IMPORTANT]  
> We provide a JS wrapper that handles this for you. The below is just a quick look at how it works.

```js
const queue = [];

const listener = RegisterCommBusListener(() => {
  listener.on("NAVIGRAPH_FunctionResult", jsonArgs => {
    const args = JSON.parse(jsonArgs);

    // When a FunctionResult is received, find the item in queue which matches the id, and resolve or reject it
    const queueItem = queue.find(m => m.id === args.id);

    if (queueItem) {
      queue.splice(queue.indexOf(queueItem), 1);
      const data = args.data;

      if (args.status === FunctionResultStatus.Success) {
        queueItem.resolve(data);
      } else {
        queueItem.reject(new Error(typeof data === "string" ? data : "Unknown error"));
      }
    }
  });
}); // RegisterCommBusListener is a function provided by sim

function getAirport(ident) {
  const id = Utils.generateGUID(); // Utils is a class provided by sim

  const args = {
    function: "GetAirport", // The name of the function being called
    id, // CallFunctions and FunctionResults are tied together with the id field
    data: {
      // The parameters of the function
      ident,
    },
  };

  listener.callWasm("NAVIGRAPH_CallFunction", JSON.stringify(args));

  return new Promise((resolve, reject) => {
    queue.push({
      id,
      resolve: response => resolve(response),
      reject: error => reject(error),
    });
  });
}

function executeSql(sql, params) {
  const id = Utils.generateGUID(); // Utils is a class provided by sim

  const args = {
    function: "ExecuteSQLQuery", // The name of the function being called
    id, // CallFunctions and FunctionResults are tied together with the id field
    data: {
      // The parameters of the function
      sql,
      params,
    },
  };

  listener.callWasm("NAVIGRAPH_CallFunction", JSON.stringify(args));

  return new Promise((resolve, reject) => {
    queue.push({
      id,
      resolve: response => resolve(response),
      reject: error => reject(error),
    });
  });
}
```
