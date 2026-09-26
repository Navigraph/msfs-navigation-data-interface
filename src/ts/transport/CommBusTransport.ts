import { NavigraphFunction, RawNavigraphEvent } from "../interface/NavigationDataInterfaceTypes";
import { NavigationDataTransport } from "./NavigationDataTransport";
import { WasmMessageRouter } from "./WasmMessageRouter";

/**
 * A transport which communicates with the Navigraph Navigation Data interface WASM gauge using the CommBus
 */
export class CommBusTransport implements NavigationDataTransport {
  private readonly listener: CommBusListener;
  private readonly router = new WasmMessageRouter();

  /**
   * Creates a new CommBusTransport
   *
   * @remarks
   * `RegisterCommBusListener` is called during construction. This means that the class must be instantiated once the function is available.
   */
  constructor() {
    this.listener = RegisterCommBusListener(() => {
      this.onRegister();
    });
  }

  public call<T>(functionName: keyof typeof NavigraphFunction, args: unknown): Promise<T> {
    const { payload, result } = this.router.createCall<T>(Utils.generateGUID(), functionName, args);

    this.listener.callWasm("NAVIGRAPH_CallFunction", payload);

    return result;
  }

  public onEvent(handler: (event: RawNavigraphEvent) => void): void {
    this.router.setEventHandler(handler);
  }

  /**
   * Registers the CommBus event listeners
   */
  private onRegister(): void {
    for (const channel of ["NAVIGRAPH_FunctionResult", "NAVIGRAPH_Event"]) {
      this.listener.on(channel, (jsonArgs: string) => this.router.handleMessage(channel, jsonArgs));
    }
  }
}
