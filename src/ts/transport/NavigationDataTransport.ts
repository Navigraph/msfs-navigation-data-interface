import { NavigraphFunction, RawNavigraphEvent } from "../interface/NavigationDataInterfaceTypes";

/**
 * A transport used by the interface to communicate with the navigation data WASM module
 */
export interface NavigationDataTransport {
  /**
   * Calls a function in the WASM module
   *
   * @param functionName - Name of the function to call
   * @param args - Data to pass to the function
   * @returns A promise that resolves with the result of the function, or rejects if the function fails
   */
  call<T>(functionName: keyof typeof NavigraphFunction, args: unknown): Promise<T>;

  /**
   * Sets the handler which receives every event emitted by the WASM module
   *
   * @param handler - Handler to be called when an event is received
   */
  onEvent(handler: (event: RawNavigraphEvent) => void): void;
}
