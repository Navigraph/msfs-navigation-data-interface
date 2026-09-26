import { CommBusTransport } from "./CommBusTransport";
import { NavigationDataTransport } from "./NavigationDataTransport";
import { StandaloneTransport, StandaloneTransportOptions } from "./StandaloneTransport";

export enum TransportMode {
  /** Use the CommBus when it is available, otherwise fall back to the standalone module */
  Auto = "Auto",
  /** Talk to the WASM gauge running in the sim, which serves the real navigation data */
  CommBus = "CommBus",
  /** Run the standalone build of the WASM module outside the sim, which serves mock data */
  Standalone = "Standalone",
}

export interface CreateTransportOptions {
  /** Which transport to create (default: {@link TransportMode.Auto}) */
  mode?: TransportMode;
  /** Options for the {@link StandaloneTransport}. Required when it may be created, i.e. in `Standalone` mode, or in `Auto` mode outside the sim. */
  standalone?: StandaloneTransportOptions;
}

/**
 * Returns whether the simulator CommBus API is available in the current environment
 */
export function isCommBusAvailable(): boolean {
  return typeof RegisterCommBusListener === "function";
}

/**
 * Creates a transport
 *
 * @param options - Which transport to create, and its options
 * @returns The created transport
 */
export function createTransport(options: CreateTransportOptions = {}): NavigationDataTransport {
  const mode = options.mode ?? TransportMode.Auto;

  if (mode === TransportMode.CommBus || (mode === TransportMode.Auto && isCommBusAvailable())) {
    return new CommBusTransport();
  }

  if (!options.standalone) {
    throw new Error(
      mode === TransportMode.Auto
        ? "The CommBus is not available. To run outside the sim, pass `standalone` options with the standalone WASM module."
        : "The standalone transport requires `standalone` options with the standalone WASM module.",
    );
  }

  return new StandaloneTransport(options.standalone);
}
