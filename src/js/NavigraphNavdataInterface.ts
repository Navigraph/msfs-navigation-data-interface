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
import { ControlledAirspace, RestrictiveAirspace } from "./types/airspace"
import { Communication } from "./types/communication"
import { DatabaseInfo } from "./types/database_info"
import { Gate } from "./types/gate"
import { GlsNavaid } from "./types/gls_navaid"
import { NdbNavaid } from "./types/ndb_navaid"
import { PathPoint } from "./types/path_point"
import { Approach, Arrival, Departure } from "./types/procedure"
import { RunwayThreshold } from "./types/runway_threshold"
import { VhfNavaid } from "./types/vhfnavaid"
import { Waypoint } from "./types/waypoint"

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
  public async execute_sql<T>(sql: string, params: string[]): Promise<T[]> {
    return await this.callWasmFunction("ExecuteSQLQuery", { sql, params })
  }

  /**
   * Downloads the navdata from the given URL to the given path
   *
   * @param url - A valid signed URL to download the navdata from
   * @param path - The path to download the navdata to
   * @returns A promise that resolves when the download is complete
   */
  public async download_navdata(url: string, path: string): Promise<void> {
    return await this.callWasmFunction("DownloadNavdata", { url, path })
  }

  /**
   * Sets the download options for all future downloads
   *
   * @param batchSize - The number of files to delete or unzip each update (default: 10). This is a performance optimization to avoid blocking the main thread for too long.
   * @returns A promise that resolves when the function is complete
   */
  public async set_download_options(batch_size: number): Promise<void> {
    return await this.callWasmFunction("SetDownloadOptions", batch_size)
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
  public async set_active_database(path: string): Promise<void> {
    return await this.callWasmFunction("SetActiveDatabase", { path })
  }

  public async get_database_info(ident: string): Promise<DatabaseInfo> {
    return await this.callWasmFunction("GetDatabaseInfo", { ident })
  }

  public async get_airport(ident: string): Promise<Airport> {
    return await this.callWasmFunction("GetAirport", { ident })
  }

  public async get_waypoints(ident: string): Promise<Waypoint[]> {
    return await this.callWasmFunction("GetWaypoints", { ident })
  }

  public async get_vhf_navaids(ident: string): Promise<VhfNavaid[]> {
    return await this.callWasmFunction("GetVhfNavaids", { ident })
  }

  public async get_ndb_navaids(ident: string): Promise<NdbNavaid[]> {
    return await this.callWasmFunction("GetNdbNavaids", { ident })
  }

  public async get_airways(ident: string): Promise<Airway[]> {
    return await this.callWasmFunction("GetAirways", { ident })
  }

  public async get_airways_at_fix(fix_ident: string, fix_icao_code: string): Promise<Airway[]> {
    return await this.callWasmFunction("GetAirwaysAtFix", { fix_ident, fix_icao_code })
  }

  public async get_airports_in_range(center: Coordinates, range: NauticalMiles): Promise<Airport[]> {
    return await this.callWasmFunction("GetAirportsInRange", { center, range })
  }

  public async get_waypoints_in_range(center: Coordinates, range: NauticalMiles): Promise<Waypoint[]> {
    return await this.callWasmFunction("GetWaypointsInRange", { center, range })
  }

  public async get_vhf_navaids_in_range(center: Coordinates, range: NauticalMiles): Promise<VhfNavaid[]> {
    return await this.callWasmFunction("GetVhfNavaidsInRange", { center, range })
  }

  public async get_ndb_navaids_in_range(center: Coordinates, range: NauticalMiles): Promise<NdbNavaid[]> {
    return await this.callWasmFunction("GetNdbNavaidsInRange", { center, range })
  }

  public async get_airways_in_range(center: Coordinates, range: NauticalMiles): Promise<Airway[]> {
    return await this.callWasmFunction("GetAirwaysInRange", { center, range })
  }

  public async get_controlled_airspaces_in_range(
    center: Coordinates,
    range: NauticalMiles,
  ): Promise<ControlledAirspace[]> {
    return await this.callWasmFunction("GetControlledAirspacesInRange", { center, range })
  }

  public async get_restrictive_airspaces_in_range(
    center: Coordinates,
    range: NauticalMiles,
  ): Promise<RestrictiveAirspace[]> {
    return await this.callWasmFunction("GetRestrictiveAirspacesInRange", { center, range })
  }

  public async get_communications_in_range(center: Coordinates, range: NauticalMiles): Promise<Communication[]> {
    return await this.callWasmFunction("GetCommunicationsInRange", { center, range })
  }

  public async get_runways_at_airport(airport_ident: string): Promise<RunwayThreshold[]> {
    return await this.callWasmFunction("GetRunwaysAtAirport", { airport_ident })
  }

  public async get_departures_at_airport(airport_ident: string): Promise<Departure[]> {
    return await this.callWasmFunction("GetDeparturesAtAirport", { airport_ident })
  }

  public async get_arrivals_at_airport(airport_ident: string): Promise<Arrival[]> {
    return await this.callWasmFunction("GetArrivalsAtAirport", { airport_ident })
  }

  public async get_approaches_at_airport(airport_ident: string): Promise<Approach[]> {
    return await this.callWasmFunction("GetApproachesAtAirport", { airport_ident })
  }

  public async get_waypoints_at_airport(airport_ident: string): Promise<Waypoint[]> {
    return await this.callWasmFunction("GetWaypointsAtAirport", { airport_ident })
  }

  public async get_ndb_navaids_at_airport(airport_ident: string): Promise<NdbNavaid[]> {
    return await this.callWasmFunction("GetNdbNavaidsAtAirport", { airport_ident })
  }

  public async get_gates_at_airport(airport_ident: string): Promise<Gate[]> {
    return await this.callWasmFunction("GetGatesAtAirport", { airport_ident })
  }

  public async get_communications_at_airport(airport_ident: string): Promise<Communication[]> {
    return await this.callWasmFunction("GetCommunicationsAtAirport", { airport_ident })
  }

  public async get_gls_navaids_at_airport(airport_ident: string): Promise<GlsNavaid[]> {
    return await this.callWasmFunction("GetGlsNavaidsAtAirport", { airport_ident })
  }

  public async get_path_points_at_airport(airport_ident: string): Promise<PathPoint[]> {
    return await this.callWasmFunction("GetPathPointsAtAirport", { airport_ident })
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
        resolve: (response: unknown) => resolve(response as T),
        reject: (error: Error) => reject(error),
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
