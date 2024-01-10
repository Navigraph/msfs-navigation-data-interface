import {
  Callback,
  CommBusMessage,
  DownloadProgressData,
  FunctionResultArgs,
  FunctionResultStatus,
  NavigraphEventType,
  NavigraphFunction,
  RawNavigraphEvent,
} from "./NavdataInterfaceTypes"
import { Airport, Airway, Coordinates, NauticalMiles } from "./types"

export class NavigraphNavdataInterface {
  private readonly listener: CommBusListener
  private queue: CommBusMessage[] = []
  private eventListeners: Callback[] = []
  private onReadyCallback: (() => void) | null = null

  private isInitialized = false

  /**
   * Creates a new NavigraphNavdataInterface
   *
   * @remarks
   * `RegisterCommBusListener` is called during construction. This means that the class must be instantiated once the function is available.
   */
  constructor() {
    this.listener = RegisterCommBusListener(() => {
      this.onRegister()
    })
  }

  /**
   * Executes a SQL query on the active database
   *
   * @remarks
   * The query must be valid SQL and must be a SELECT query. The query must not specify a special result format.
   *
   * @param sql - SQL query to execute
   * @returns A promise that resolves with the result of the query
   */
  public async executeSql<T>(sql: string, params: string[]): Promise<T[]> {
    return await this.callWasmFunction("ExecuteSQLQuery", { sql, params })
  }

  /**
   * Downloads the navdata from the given URL to the given path
   *
   * @param url - A valid signed URL to download the navdata from
   * @param path - The path to download the navdata to
   * @returns A promise that resolves when the download is complete
   */
  public async downloadNavdata(url: string, path: string): Promise<void> {
    return await this.callWasmFunction("DownloadNavdata", { url, path })
  }

  /**
   * Sets the download options for all future downloads
   *
   * @param batchSize - The number of files to delete or unzip each update (default: 10). This is a performance optimization to avoid blocking the main thread for too long.
   * @returns A promise that resolves when the function is complete
   */
  public async setDownloadOptions(batchSize: number): Promise<void> {
    return await this.callWasmFunction("SetDownloadOptions", batchSize)
  }

  /**
   * Sets the active DFD database to the one at the given path
   *
   * @remarks
   * The path must be a valid path to a folder that contains a DFD navdata database.
   *
   * @param path - The path to the folder that contains the DFD navdata
   * @returns A promise that resolves when the function is complete
   */
  public async setActiveDatabase(path: string): Promise<void> {
    return await this.callWasmFunction("SetActiveDatabase", { path })
  }

  public async getAirport(ident: string): Promise<Airport> {
    return await this.callWasmFunction("GetAirport", { ident })
  }

  public async getAirportsInRange(center: Coordinates, range: NauticalMiles): Promise<Airport[]> {
    return await this.callWasmFunction("GetAirportsInRange", { center, range })
  }

  public async getAirways(ident: string): Promise<Airway[]> {
    return await this.callWasmFunction("GetAirways", { ident })
  }

  public async getAirwaysInRange(center: Coordinates, range: NauticalMiles): Promise<Airway[]> {
    return await this.callWasmFunction("GetAirwaysInRange", { center, range })
  }

  private recursiveToCamel(item: unknown): unknown {
    if (Array.isArray(item)) {
      return item.map((el: unknown) => this.recursiveToCamel(el))
    } else if (typeof item === "function" || item !== Object(item)) {
      return item
    }
    return Object.fromEntries(
      Object.entries(item as Record<string, unknown>).map(([key, value]: [string, unknown]) => [
        key.replace(/([-_][a-z])/gi, c => c.toUpperCase().replace(/[-_]/g, "")),
        this.recursiveToCamel(value),
      ]),
    )
  }

  /**
   * Call a function in the WASM module
   *
   * @param name - Name of the function to call
   * @param data - Data to pass to the function
   * @returns A promise that resolves when the function returns
   */
  private async callWasmFunction<T = unknown>(name: keyof typeof NavigraphFunction, data: unknown): Promise<T> {
    const id = Utils.generateGUID()

    const args = {
      function: name,
      id,
      data,
    }

    this.listener.callWasm("NAVIGRAPH_CallFunction", JSON.stringify(args))

    return new Promise((resolve, reject) => {
      this.queue.push({
        id,
        resolve: (response: unknown) => resolve(this.recursiveToCamel(response) as T),
        reject: (error: Error) => reject(this.recursiveToCamel(error)),
      })
    })
  }

  /**
   * Registers the event listeners for the interface
   */
  private onRegister(): void {
    this.listener.on("NAVIGRAPH_FunctionResult", (jsonArgs: string) => {
      const args = JSON.parse(jsonArgs) as FunctionResultArgs
      const id = args.id

      // Find the function call in the queue and resolve/reject it
      const message = this.queue.find(m => m.id === id)
      if (message) {
        this.queue.splice(this.queue.indexOf(message), 1)
        const data = args.data
        if (args.status === FunctionResultStatus.Success) {
          message.resolve(data)
        } else {
          message.reject(new Error(typeof data === "string" ? data : "Unknown error"))
        }
      }
    })

    this.listener.on("NAVIGRAPH_Event", (jsonArgs: string) => {
      const args = JSON.parse(jsonArgs) as RawNavigraphEvent

      // If this is the heartbeat event, set the interface as initialized
      if (args.event === NavigraphEventType.Heartbeat && !this.isInitialized) {
        this.isInitialized = true
        if (this.onReadyCallback) {
          this.onReadyCallback()
        }
      }

      // Call all callbacks for the event
      if (args.event in NavigraphEventType) {
        const callbacks = this.eventListeners.filter(cb => cb.event === args.event)
        callbacks.forEach(cb => cb.callback(args.data))
      }
    })
  }

  public onEvent(event: NavigraphEventType.Heartbeat, callback: () => void): void
  public onEvent(event: NavigraphEventType.DownloadProgress, callback: (data: DownloadProgressData) => void): void

  /**
   * Registers a callback to be called when an event is received
   *
   * @param event - Event to listen to
   * @param callback - Callback to be called when the event is received
   */
  public onEvent<T = unknown>(event: NavigraphEventType, callback: (data: T) => void): void {
    const cb: Callback<T> = {
      event,
      callback,
    }
    this.eventListeners.push(cb as Callback)
  }

  /**
   * Registers a callback to be called when the interface is ready
   *
   * @param callback - Callback to be called when the interface is ready
   */
  public onReady(callback: () => void): void {
    this.onReadyCallback = callback
  }

  /**
   * Returns whether the interface is ready
   *
   * @returns Whether the interface is ready
   */
  public getIsInitialized(): boolean {
    return this.isInitialized
  }
}
