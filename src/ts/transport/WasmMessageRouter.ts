import {
  CommBusMessage,
  FunctionResultArgs,
  FunctionResultStatus,
  NavigraphFunction,
  RawNavigraphEvent,
} from "../interface/NavigationDataInterfaceTypes";

/**
 * Tracks pending function calls and routes the messages sent by the WASM module (`NAVIGRAPH_FunctionResult` and `NAVIGRAPH_Event`).
 *
 * @remarks
 * The payloads are identical whether the module runs in the sim (CommBus) or standalone, so this is shared by all transports.
 */
export class WasmMessageRouter {
  private queue: CommBusMessage[] = [];
  private eventHandler: ((event: RawNavigraphEvent) => void) | null = null;

  /**
   * Registers a function call, returning the `NAVIGRAPH_CallFunction` payload to send and a promise for its result
   *
   * @param id - Unique ID of the call, used to match it with its result
   * @param functionName - Name of the function to call
   * @param args - Data to pass to the function
   */
  public createCall<T>(
    id: string,
    functionName: keyof typeof NavigraphFunction,
    args: unknown,
  ): { payload: string; result: Promise<T> } {
    const result = new Promise<T>((resolve, reject) => {
      this.queue.push({
        id,
        resolve: (response: unknown) => resolve(response as T),
        reject: (error: Error) => reject(error),
      });
    });

    return { payload: JSON.stringify({ function: functionName, id, data: args }), result };
  }

  public setEventHandler(handler: (event: RawNavigraphEvent) => void): void {
    this.eventHandler = handler;
  }

  /**
   * Routes a message sent by the WASM module
   *
   * @param channel - Channel of the message, such as `NAVIGRAPH_FunctionResult`
   * @param jsonArgs - Serialized message
   */
  public handleMessage(channel: string, jsonArgs: string): void {
    switch (channel) {
      case "NAVIGRAPH_FunctionResult":
        this.handleFunctionResult(jsonArgs);
        break;
      case "NAVIGRAPH_Event":
        this.eventHandler?.(JSON.parse(jsonArgs) as RawNavigraphEvent);
        break;
    }
  }

  /**
   * Rejects all pending calls, e.g. when the transport fails
   *
   * @param error - Error to reject the calls with
   */
  public rejectAll(error: Error): void {
    const pending = this.queue;
    this.queue = [];
    pending.forEach(m => m.reject(error));
  }

  private handleFunctionResult(jsonArgs: string): void {
    const args = JSON.parse(jsonArgs) as FunctionResultArgs;
    const id = args.id;

    // Find the function call in the queue and resolve/reject it
    const message = this.queue.find(m => m.id === id);
    if (message) {
      this.queue.splice(this.queue.indexOf(message), 1);
      const data = args.data;
      if (args.status === FunctionResultStatus.Success) {
        message.resolve(data);
      } else {
        message.reject(new Error(typeof data === "string" ? data : "Unknown error"));
      }
    }
  }
}
