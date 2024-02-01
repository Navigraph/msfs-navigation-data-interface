import {
  Airport,
  Airway,
  Approach,
  Arrival,
  Communication,
  ControlledAirspace,
  Coordinates,
  DatabaseInfo,
  Departure,
  Gate,
  GlsNavaid,
  NauticalMiles,
  NdbNavaid,
  PathPoint,
  RestrictiveAirspace,
  RunwayThreshold,
  VhfNavaid,
  Waypoint,
} from "../types"
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

/**
 * A TS wrapper class used for interfacing with the Navigraph Navigation Data interface WASM gauge using the CommBus
 */
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

  /**
   * Gets information about the currently active database
   */
  public async get_database_info(ident: string): Promise<DatabaseInfo> {
    return await this.callWasmFunction("GetDatabaseInfo", { ident })
  }

  /**
   * Gets data for an airport
   * @param ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the airport data, or rejects if the airport does not exist
   */
  public async get_airport(ident: string): Promise<Airport> {
    return await this.callWasmFunction("GetAirport", { ident })
  }

  /**
   * Gets a list of waypoints
   * @param ident - The identifier to get the waypoints by
   * @returns A promise that resolves with the list of waypoints
   */
  public async get_waypoints(ident: string): Promise<Waypoint[]> {
    return await this.callWasmFunction("GetWaypoints", { ident })
  }

  /**
   * Gets a list of vhf navaids
   * @param ident - The identifier to get the vhf navaids by
   * @returns A promise that resolves with the list of vhf navaids
   */
  public async get_vhf_navaids(ident: string): Promise<VhfNavaid[]> {
    return await this.callWasmFunction("GetVhfNavaids", { ident })
  }

  /**
   * Gets a list of ndb navaids
   * @param ident - The identifier to get the ndb navaids by
   * @returns A promise that resolves with the list of ndb navaids
   */
  public async get_ndb_navaids(ident: string): Promise<NdbNavaid[]> {
    return await this.callWasmFunction("GetNdbNavaids", { ident })
  }

  /**
   * Gets a list of airways
   * @param ident - The identifier to get the airways by
   * @returns A promise that resolves with the list of airways
   */
  public async get_airways(ident: string): Promise<Airway[]> {
    return await this.callWasmFunction("GetAirways", { ident })
  }

  /**
   * Gets a list of airways which pass through the given fix
   * @param fix_ident - The identifier of the fix to get the airways by
   * @param fix_icao_code - The ICAO code of the fix to get the airways by
   * @returns A promise that resolves with the list of airways
   */
  public async get_airways_at_fix(fix_ident: string, fix_icao_code: string): Promise<Airway[]> {
    return await this.callWasmFunction("GetAirwaysAtFix", { fix_ident, fix_icao_code })
  }

  /**
   * Gets all airports within a given range circle around a given point
   * @param center - The center of the range circle
   * @param range - The radius of the range circle (Nautical miles)
   * @returns A promise that resolves with the list of airports
   */
  public async get_airports_in_range(center: Coordinates, range: NauticalMiles): Promise<Airport[]> {
    return await this.callWasmFunction("GetAirportsInRange", { center, range })
  }

  /**
   * Gets all waypoints within a given range circle around a given point
   * @param center - The center of the range circle
   * @param range - The radius of the range circle (Nautical miles)
   * @returns A promise that resolves with the list of waypoints
   */
  public async get_waypoints_in_range(center: Coordinates, range: NauticalMiles): Promise<Waypoint[]> {
    return await this.callWasmFunction("GetWaypointsInRange", { center, range })
  }

  /**
   * Gets all vhf navaids within a given range circle around a given point
   * @param center - The center of the range circle
   * @param range - The radius of the range circle (Nautical miles)
   * @returns A promise that resolves with the list of vhf navaids
   */
  public async get_vhf_navaids_in_range(center: Coordinates, range: NauticalMiles): Promise<VhfNavaid[]> {
    return await this.callWasmFunction("GetVhfNavaidsInRange", { center, range })
  }

  /**
   * Gets all ndb navaids within a given range circle around a given point
   * @param center - The center of the range circle
   * @param range - The radius of the range circle (Nautical miles)
   * @returns A promise that resolves with the list of ndb navaids
   */
  public async get_ndb_navaids_in_range(center: Coordinates, range: NauticalMiles): Promise<NdbNavaid[]> {
    return await this.callWasmFunction("GetNdbNavaidsInRange", { center, range })
  }

  /**
   * Gets all airways which have a fix which falls within a given range circle around a given point
   * @param center - The center of the range circle
   * @param range - The radius of the range circle (Nautical miles)
   * @returns A promise that resolves with the list of airways
   */
  public async get_airways_in_range(center: Coordinates, range: NauticalMiles): Promise<Airway[]> {
    return await this.callWasmFunction("GetAirwaysInRange", { center, range })
  }

  /**
   * Gets all controlled airspaces which have an edge vertex which falls within a given range circle around a given point
   * @param center - The center of the range circle
   * @param range - The radius of the range circle (Nautical miles)
   * @returns A promise that resolves with the list of controlled airspaces
   */
  public async get_controlled_airspaces_in_range(
    center: Coordinates,
    range: NauticalMiles,
  ): Promise<ControlledAirspace[]> {
    return await this.callWasmFunction("GetControlledAirspacesInRange", { center, range })
  }

  /**
   * Gets all restrictive airspaces which have an edge vertex which falls within a given range circle around a given point
   * @param center - The center of the range circle
   * @param range - The radius of the range circle (Nautical miles)
   * @returns A promise that resolves with the list of restrictive airspaces
   */
  public async get_restrictive_airspaces_in_range(
    center: Coordinates,
    range: NauticalMiles,
  ): Promise<RestrictiveAirspace[]> {
    return await this.callWasmFunction("GetRestrictiveAirspacesInRange", { center, range })
  }

  /**
   * Gets all communications (airport and enroute) which have their station fall within a given range circle around a given point
   * @param center - The center of the range circle
   * @param range - The radius of the range circle (Nautical miles)
   * @returns A promise that resolves with the list of communications
   */
  public async get_communications_in_range(center: Coordinates, range: NauticalMiles): Promise<Communication[]> {
    return await this.callWasmFunction("GetCommunicationsInRange", { center, range })
  }

  /**
   * Gets all runways which serve an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of runways
   */
  public async get_runways_at_airport(airport_ident: string): Promise<RunwayThreshold[]> {
    return await this.callWasmFunction("GetRunwaysAtAirport", { airport_ident })
  }

  /**
   * Gets all departure procedures which serve an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of departures
   */
  public async get_departures_at_airport(airport_ident: string): Promise<Departure[]> {
    return await this.callWasmFunction("GetDeparturesAtAirport", { airport_ident })
  }

  /**
   * Gets all arrival procedures which serve an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of arrivals
   */
  public async get_arrivals_at_airport(airport_ident: string): Promise<Arrival[]> {
    return await this.callWasmFunction("GetArrivalsAtAirport", { airport_ident })
  }

  /**
   * Gets all approach procedures which serve an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of approaches
   */
  public async get_approaches_at_airport(airport_ident: string): Promise<Approach[]> {
    return await this.callWasmFunction("GetApproachesAtAirport", { airport_ident })
  }

  /**
   * Gets all terminal waypoints which are affiliated with an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of waypoints
   */
  public async get_waypoints_at_airport(airport_ident: string): Promise<Waypoint[]> {
    return await this.callWasmFunction("GetWaypointsAtAirport", { airport_ident })
  }

  /**
   * Gets all ndb navaids which are affiliated with an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of ndb navaids
   */
  public async get_ndb_navaids_at_airport(airport_ident: string): Promise<NdbNavaid[]> {
    return await this.callWasmFunction("GetNdbNavaidsAtAirport", { airport_ident })
  }

  /**
   * Gets all gates which are at an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of gates
   */
  public async get_gates_at_airport(airport_ident: string): Promise<Gate[]> {
    return await this.callWasmFunction("GetGatesAtAirport", { airport_ident })
  }

  /**
   * Gets all communications which are at an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of communications
   */
  public async get_communications_at_airport(airport_ident: string): Promise<Communication[]> {
    return await this.callWasmFunction("GetCommunicationsAtAirport", { airport_ident })
  }

  /**
   * Gets all gls navaids which serve an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of gls navaids
   */
  public async get_gls_navaids_at_airport(airport_ident: string): Promise<GlsNavaid[]> {
    return await this.callWasmFunction("GetGlsNavaidsAtAirport", { airport_ident })
  }

  /**
   * Gets all path points which serve an airport
   * @param airport_ident - The 4 letter identifier of the airport
   * @returns A promise that resolves with the list of path points
   */
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
