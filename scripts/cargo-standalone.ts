import { $ } from "bun";
import { copyFileSync, mkdirSync } from "node:fs";
import { join, resolve } from "node:path";

/// Builds the standalone WASM module, which runs outside the sim and serves mock navigation data.
///
/// The mock navigation data is generated inside the container, right before the build, and embedded into the module.
/// Usage: `bun run build:wasm:standalone [AIRPORT ...]` (airports default to those in scripts/generate-mock-navdata.ts)

/// The docker image name
const IMAGE_NAME = "navigation-data-interface-standalone-build";

/// Docker volume used to cache the cargo registry between builds
const REGISTRY_VOLUME = "navigation-data-interface-cargo-registry";

/// Where the mock navigation data is generated, relative to the workspace root (inside the gitignored targets folder)
const MOCK_DATA_DIR = "targets/standalone/mock-data";

const workspaceRoot = resolve(import.meta.dir, "..");

/// Airports to generate mock data for, passed through to the generator
const airports = process.argv.slice(2).filter(a => /^[A-Za-z0-9]{3,4}$/.test(a));

// Ensure docker is installed and available
await $`docker ps`.quiet().catch(() => {
  console.error("[-] Docker is not installed or not running");
  process.exit(1);
});

// Ensure image is built
await $`docker image inspect ${IMAGE_NAME}:latest`.quiet().catch(async () => {
  const dockerfilePath = resolve(workspaceRoot, "Dockerfile.standalone");
  console.info(`[*] Building '${IMAGE_NAME}' image from ${dockerfilePath}`);
  await $`docker build -t ${IMAGE_NAME} -f ${dockerfilePath} ${workspaceRoot}`;
});

console.info("[*] Generating mock navigation data and building standalone module");

await $`docker run \
  --rm -t \
  --name msfs-standalone-wasm-builder \
  -v ${workspaceRoot}:/workspace \
  -v ${REGISTRY_VOLUME}:/usr/local/cargo/registry \
  -w /workspace \
  -e CARGO_TARGET_DIR=/workspace/targets/standalone \
  -e NAVIGRAPH_MOCK_DATA_DIR=/workspace/${MOCK_DATA_DIR} \
  ${IMAGE_NAME} \
    bash -c ${`bun ./scripts/generate-mock-navdata.ts --out ${MOCK_DATA_DIR} ${airports.join(" ")} && \
cargo build -p msfs-navigation-data-interface --release --target wasm32-wasip1 --no-default-features --features standalone`}`.catch(
  (err: { exitCode?: number }) => {
    console.error(`[-] Error building standalone module: ${err.exitCode}`);
    process.exit(1);
  },
);

const builtModule = join(workspaceRoot, "targets/standalone/wasm32-wasip1/release/msfs_navigation_data_interface.wasm");
const outDir = resolve(workspaceRoot, "dist/standalone");
mkdirSync(outDir, { recursive: true });
copyFileSync(builtModule, join(outDir, "msfs_navigation_data_interface.wasm"));

console.info(`[+] Standalone module written to ${outDir}`);
