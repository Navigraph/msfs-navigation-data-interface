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
  - `standalone-demo/` runs the [standalone module](#running-outside-the-sim-standalone-mode) outside the sim, as a script and as a NestJS HTTP API with Swagger UI
- `scripts/` includes the build scripts, including the [standalone build](#running-outside-the-sim-standalone-mode) and its mock data generator
- `src/`
  - `ts` includes source code for the JS interface for interfacing with the WASM module
    - `transport/` includes the [transports](#using-the-js-interface-with-a-transport) used to reach the WASM module (CommBus in the sim, or the standalone module)
  - `wasm` includes the Rust source code for the WASM module which handles the downloading of the database file, and interfacing with the database
    - `platform/` includes the [platform adapters](#platforms) (MSFS or standalone), selected at compile time

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
       "developer": "YOUR NAME/COMPANY HERE",
       "product": "YOUR PRODUCT NAME HERE"
     }
   }
   ```

## Dealing with Bundled Navigation Data

If you bundle outdated navigation data in your aircraft and you want this module to handle updating it for users with subscriptions, place the navigation data into the `Navigraph/BundledData` directory in `PackageSources`. You can see an example [here](example/aircraft/PackageSources/Navigraph/BundledData/)

The navigation data interface will automatically use this database by default, making it immediately available on startup.

## Where is the Navigation Data Stored?

The default location for navigation data is `work/NavigationData`.

## Building the Sample Aircraft

> [!NOTE]  
> This project is meant to work in MSFS2020 and MSFS2024.

> [!IMPORTANT]  
> Before building, make sure you have properly created and set an `.env` file in `example/gauge`! An example can be found in the `.env.example` file in that directory. Replace with your credentials.

> [!IMPORTANT]  
> Create a `.env` file in the root of this repository, containing a `SENTRY_URL` variable. Provide your own DSN, or leave it empty.

1. Download and install [Bun](https://bun.sh/docs/installation).
2. Open this repository in a terminal.
3. Run `bun i` the first time you build, in order to install dependencies.
4. Run `bun run build:example`. This command will [build](#building-the-wasm-module-yourself) the wasm module, [build](#building-the-gauge-yourself) the gauge and [copy](#building-the-wasm-module-yourself) the module to the aircraft `panel` folder.
5. Open the `example/aircraft/NavigationDataInterfaceAircraftProject.xml` file in the simulator and build the package.

## Building the WASM Module Yourself

> [!IMPORTANT]  
> Create a `.env` file in the root of this repository, containing a `SENTRY_URL` variable. Provide your own DSN, or leave it empty.

1. Run `bun run build:wasm` at the root of the repository (requires Docker)
   - This will take a while to download and build the first time, but subsequent runs will be quicker.
   - The WASM module will be compatible for MSFS 2020 & 2024.
2. Run `bun run copy:wasm` command to copy the wasm module or do it manually by copying the wasm module in the aircraft `panel` folder.

## Building the Gauge Yourself

> [!IMPORTANT]  
> Before building, make sure you have properly created and set an `.env` file in `example/gauge`! An example can be found in the `.env.example` file in that directory. Replace with your credentials.

1. Change directory to [`example/gauge`](example/gauge/) using `cd example/gauge`
2. Run `bun run build` to build into the `PackageSources` folder of the aircraft sample (or `bun run dev` to build into the `Packages` folder of the aircraft and listen to changes in the source).

## Running Outside the Sim (Standalone Mode)

The WASM module can also be built as a **standalone** module, which runs outside the simulator (in a browser, or in Bun/Node) and serves mock navigation data. This lets you develop and test instruments and tools against the real interface, without starting MSFS.

The standalone module runs the same Rust core as the sim build: every function (including `ExecuteSQLQuery`) goes through the same dispatch and the same SQL queries. Only the platform-specific parts are swapped out, and the navigation data is a mock database embedded in the module.

### Platforms

The Rust code is split into a shared core and a platform adapter, selected at compile time with a Cargo feature:

```
                    Rust core
      (database, SQL queries, function dispatch, events)
                        │
                 trait Platform
                 /            \
        MsfsPlatform       StandalonePlatform
        CommBus            host import (navigraph.send_message)
        work folder,       embedded mock database
        bundled data,      (in-memory SQLite)
        network download   simulated download
```

| Feature          | Build command                   | Used for                                |
| ---------------- | ------------------------------- | --------------------------------------- |
| `msfs` (default) | `bun run build:wasm`            | The gauge in MSFS 2020 & 2024           |
| `standalone`     | `bun run build:wasm:standalone` | Running outside the sim, with mock data |

The two features are mutually exclusive. The platform trait lives in [`src/wasm/src/platform`](src/wasm/src/platform/mod.rs), and the core only talks to the platform through it.

| Behaviour                        | `msfs`                                                   | `standalone`                                                             |
| -------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------ |
| Messages to JS                   | CommBus                                                  | `navigraph.send_message`, imported from the host                         |
| Navigation data                  | `work/NavigationData` (bundled or downloaded)            | Mock database, embedded in the module                                    |
| `DownloadNavigationData`         | Downloads and installs the data from the URL             | Sends `DownloadProgress` events, then reloads the mock data (no network) |
| `GetNavigationDataInstallStatus` | Installed cycle, and latest cycle from the Navigraph API | Cycle of the mock data                                                   |

### Building the Standalone Module

1. Make sure Docker is running.
2. Run `bun run build:wasm:standalone` at the root of the repository.
   - The first run builds the `navigation-data-interface-standalone-build` image from [`Dockerfile.standalone`](Dockerfile.standalone). It does not need the MSFS SDK.
   - The mock navigation data is generated inside the container, right before the build (see [Mock Navigation Data](#mock-navigation-data)).
3. The module is written to `dist/standalone/msfs_navigation_data_interface.wasm`.

> [!IMPORTANT]  
> The standalone module must be built through `bun run build:wasm:standalone`. Running `cargo build --no-default-features --features standalone` directly fails, as the mock navigation data only exists in the build container.

### Mock Navigation Data

The mock navigation data is generated by [`scripts/generate-mock-navdata.ts`](scripts/generate-mock-navdata.ts), inside the standalone build container. It is never committed, and is written to `targets/standalone/mock-data`.

It is a subset of the [example bundled database](example/aircraft/PackageSources/Navigraph/BundledData/), so it has exactly the same schema as real navigation data:

- The selected airports, with all their procedures, runways, gates, communications, etc. By default these are `MMUN`, `MMMD` and `MSLP`.
- All other airports (with their runways) within about 2° of the selected airports, so range queries return realistic results.
- Enroute waypoints, navaids, holdings and communications in the same area, plus the airways, airspaces and FIRs passing through it.

To generate the mock data for other airports, pass them to the build (they must exist in the example database):

```sh
bun run build:wasm:standalone MMMX MMGL
```

### Using the JS Interface with a Transport

The JS interface talks to the WASM module through a transport, found in [`src/ts/transport`](src/ts/transport):

- `CommBusTransport`: talks to the gauge in the sim over the CommBus.
- `StandaloneTransport`: loads the standalone module, drives it (the equivalent of sim frames), and passes calls to it. It works in browsers and in Bun/Node.

By default, the interface uses the CommBus when it is available. To run outside the sim, pass the standalone module:

```ts
import { NavigraphNavigationDataInterface, TransportMode } from "@navigraph/msfs-navigation-data-interface";

// In the sim (default): uses the CommBus
const simInterface = new NavigraphNavigationDataInterface();

// Outside the sim: runs the standalone module, serving mock data
const navigationDataInterface = new NavigraphNavigationDataInterface({
  mode: TransportMode.Standalone,
  standalone: {
    // A URL to fetch, the module bytes, a compiled WebAssembly.Module, or a fetch Response
    wasm: "/msfs_navigation_data_interface.wasm",
  },
});

navigationDataInterface.onReady(async () => {
  const airport = await navigationDataInterface.get_airport("MMUN");
});
```

For runnable examples, see [`example/standalone-demo`](example/standalone-demo): a script (`bun run demo:standalone`), and a NestJS HTTP API with Swagger UI serving the mock data (`bun run demo:standalone:serve`).

The available modes are:

- `TransportMode.Auto` (default): uses the CommBus when `RegisterCommBusListener` is available. Otherwise, uses the standalone module if `standalone` options are passed, and throws if they are not.
- `TransportMode.CommBus`: always uses the CommBus.
- `TransportMode.Standalone`: always uses the standalone module (requires `standalone` options).

You can also pass your own transport, by implementing the `NavigationDataTransport` interface (`call` and `onEvent`):

```ts
const navigationDataInterface = new NavigraphNavigationDataInterface(myTransport);
```

### Hosting the Standalone Module Yourself

If you are not using the JS interface, the standalone module can be driven directly. It uses the same payloads and channels as the [CommBus events](#interfacing-with-the-gauge-manually):

- Exports:
  - `navigraph_alloc(len) -> ptr` allocates a buffer in the module memory.
  - `navigraph_call_function(ptr, len)` queues a function call. The buffer holds a UTF-8 `NAVIGRAPH_CallFunction` payload, and ownership passes to the module.
  - `navigraph_dealloc(ptr, len)` frees a buffer which was not passed to `navigraph_call_function`.
  - `navigraph_update()` runs queued functions and sends the heartbeat. Call it regularly, like a sim frame (e.g. every 16ms).
- Imports:
  - `navigraph.send_message(channel_ptr, channel_len, data_ptr, data_len)` receives `NAVIGRAPH_FunctionResult` and `NAVIGRAPH_Event` messages (UTF-8). Copy the data before returning, and avoid calling back into the module from it.
  - `wasi_snapshot_preview1`, as the module targets `wasm32-wasip1`. Only stdout/stderr, clocks and randomness are needed; [`StandaloneTransport`](src/ts/transport/StandaloneTransport.ts) contains a minimal implementation.

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
