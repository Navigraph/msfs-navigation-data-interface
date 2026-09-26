import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { NavigraphNavigationDataInterface, StandaloneTransport } from "./navigraph";

/// The standalone module, as written by `bun run build:wasm:standalone`
export const DEFAULT_WASM_PATH = fileURLToPath(
  new URL("../../../dist/standalone/msfs_navigation_data_interface.wasm", import.meta.url),
);

export interface LoadedNavigationData {
  navigationDataInterface: NavigraphNavigationDataInterface;
  /** Drives the module on an interval, which keeps the process alive until it is disposed */
  transport: StandaloneTransport;
}

/**
 * Loads the standalone module and waits until it is ready (its first heartbeat)
 *
 * @param wasmPath - Path of the standalone module
 * @param timeoutMs - Milliseconds to wait for the module to become ready
 */
export function loadNavigationData(wasmPath = DEFAULT_WASM_PATH, timeoutMs = 5000): Promise<LoadedNavigationData> {
  if (!existsSync(wasmPath)) {
    return Promise.reject(
      new Error(`${wasmPath} not found. Run \`bun run build:wasm:standalone\` at the root of the repository first.`),
    );
  }

  const transport = new StandaloneTransport({ wasm: readFileSync(wasmPath) });
  const navigationDataInterface = new NavigraphNavigationDataInterface(transport);

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      transport.dispose();
      reject(new Error(`The standalone module did not become ready within ${timeoutMs}ms`));
    }, timeoutMs);

    navigationDataInterface.onReady(() => {
      clearTimeout(timeout);
      resolve({ navigationDataInterface, transport });
    });
  });
}
