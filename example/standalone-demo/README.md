# Standalone Demo

Runs the standalone build of the WASM module outside the sim, through the JS interface (`NavigraphNavigationDataInterface` with a `StandaloneTransport`). It comes in two forms:

- A **script** (`index.ts`) which prints some mock navigation data.
- An **HTTP API** (`server.ts`, built with NestJS) which serves the mock navigation data, with an OpenAPI spec and Swagger UI.

Both serve mock data: only the airports the mock data was generated for (`MMUN`, `MMMD` and `MSLP` by default) and their surroundings are available. See [Running Outside the Sim](../../README.md#running-outside-the-sim-standalone-mode) for how the mock data is built.

## Prerequisites

Build the standalone module (requires Docker) at the root of the repository:

```sh
bun run build:wasm:standalone
```

This writes `dist/standalone/msfs_navigation_data_interface.wasm`, which the demo loads. The demo imports the JS interface from source (`src/ts`), so the package doesn't need to be built.

## Script

```sh
bun run demo:standalone          # from the root, or
cd example/standalone-demo
bun start [AIRPORT]              # defaults to MMUN
```

## HTTP API

```sh
bun run demo:standalone:serve    # from the root, or
cd example/standalone-demo
bun run serve
```

Then open http://localhost:3000/docs for the Swagger UI. The spec, generated from the controllers by `@nestjs/swagger`, is served at `/openapi.json`.

| Environment variable   | Default                                                  | Description                   |
| ---------------------- | -------------------------------------------------------- | ----------------------------- |
| `PORT`                 | `3000`                                                   | Port to listen on             |
| `NAVIGRAPH_WASM_PATH`  | `dist/standalone/msfs_navigation_data_interface.wasm`    | Path of the standalone module |

### Endpoints

All API endpoints are under `/api`. Identifiers are case insensitive.

| Endpoint                                   | Description                                         |
| ------------------------------------------ | --------------------------------------------------- |
| `GET /health`                              | Health check                                        |
| `GET /api/database`                        | Cycle and install status of the navigation data     |
| `GET /api/airports?lat=&long=&range=`      | Airports within `range` NM (max 500) of a point     |
| `GET /api/airports/{ident}`                | An airport                                          |
| `GET /api/airports/{ident}/runways`        | Runways of an airport                               |
| `GET /api/airports/{ident}/departures`     | Departures (SIDs)                                   |
| `GET /api/airports/{ident}/arrivals`       | Arrivals (STARs)                                    |
| `GET /api/airports/{ident}/approaches`     | Approaches                                          |
| `GET /api/airports/{ident}/gates`          | Gates                                               |
| `GET /api/airports/{ident}/communications` | Communication frequencies                           |
| `GET /api/waypoints/{ident}`               | Waypoints with an identifier (they're not unique)   |
| `GET /api/navaids/vhf/{ident}`             | VHF navaids with an identifier                      |
| `GET /api/navaids/ndb/{ident}`             | NDB navaids with an identifier                      |
| `GET /api/airways/{ident}`                 | Airways with an identifier                          |

```sh
curl http://localhost:3000/api/airports/MMUN/runways
curl "http://localhost:3000/api/airports?lat=21.04&long=-86.87&range=50"
```

Errors use the standard Nest format, `{ "statusCode": 404, "error": "Not Found", "message": "..." }`, with status `400` for invalid input (e.g. a malformed identifier or a missing query parameter), `404` when nothing matches, and `500` for unexpected failures of the module.

### Structure

The business logic is kept apart from HTTP, so each can be tested on its own:

```
server.ts                   Entry point: loads the module, starts the app
src/
  loadNavigationData.ts     Loads the standalone module and waits until it's ready
  NavigationDataService.ts  Business logic: validation, normalization, not found checks. Plain TS, no Nest or HTTP.
  errors.ts                 ValidationError, NotFoundError
  controllers.ts            All the controllers: map routes to service calls, and document them for Swagger
  app.module.ts             The Nest module, the filter mapping service errors to 400/404, and Swagger setup
```

The service depends on the `NavigationDataSource` type, the subset of the JS interface it uses, and `AppModule.register(source)` creates it. This way tests can replace the WASM module with an in-memory fake.

## Tests

```sh
cd example/standalone-demo
bun test
```

- `tests/NavigationDataService.test.ts`: the business logic, against an in-memory fake source.
- `tests/app.test.ts`: the HTTP layer (routes, status codes, OpenAPI spec and Swagger UI), against the same fake.
- `tests/integration.test.ts`: the service, and every example in the generated OpenAPI spec, against the real standalone module. Skipped when the module hasn't been built.
