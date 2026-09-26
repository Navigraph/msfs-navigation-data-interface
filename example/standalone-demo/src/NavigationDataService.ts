import type {
  Airport,
  Airway,
  Approach,
  Arrival,
  Communication,
  DatabaseInfo,
  Departure,
  Gate,
  NavigationDataStatus,
  NavigraphNavigationDataInterface,
  NdbNavaid,
  RunwayThreshold,
  VhfNavaid,
  Waypoint,
} from "./navigraph";
import { NotFoundError, ValidationError } from "./errors";

/** The parts of the JS interface used by the service, so it can be replaced in tests */
export type NavigationDataSource = Pick<
  NavigraphNavigationDataInterface,
  | "execute_sql"
  | "get_database_info"
  | "get_navigation_data_install_status"
  | "get_airport"
  | "get_airports_in_range"
  | "get_runways_at_airport"
  | "get_departures_at_airport"
  | "get_arrivals_at_airport"
  | "get_approaches_at_airport"
  | "get_gates_at_airport"
  | "get_communications_at_airport"
  | "get_waypoints"
  | "get_vhf_navaids"
  | "get_ndb_navaids"
  | "get_airways"
>;

export interface DatabaseSummary {
  info: DatabaseInfo;
  status: NavigationDataStatus;
}

/** The largest range accepted by range queries, to keep responses reasonably sized */
export const MAX_RANGE_NM = 500;

const AIRPORT_IDENT = /^[A-Z0-9]{3,4}$/;
const FIX_IDENT = /^[A-Z0-9]{1,5}$/;

/**
 * The business logic of the API: validates and normalizes input, and resolves it against the navigation data.
 * Knows nothing about HTTP.
 */
export class NavigationDataService {
  constructor(private readonly source: NavigationDataSource) {}

  public async getDatabase(): Promise<DatabaseSummary> {
    const [info, status] = await Promise.all([
      this.source.get_database_info(),
      this.source.get_navigation_data_install_status(),
    ]);
    return { info, status };
  }

  public async getAirport(ident: string): Promise<Airport> {
    return this.source.get_airport(await this.requireAirport(ident));
  }

  public async getAirportsInRange(lat: number, long: number, range: number): Promise<Airport[]> {
    if (!Number.isFinite(lat) || lat < -90 || lat > 90) {
      throw new ValidationError("lat must be a number between -90 and 90");
    }
    if (!Number.isFinite(long) || long < -180 || long > 180) {
      throw new ValidationError("long must be a number between -180 and 180");
    }
    if (!Number.isFinite(range) || range <= 0 || range > MAX_RANGE_NM) {
      throw new ValidationError(`range must be a number greater than 0 and at most ${MAX_RANGE_NM}`);
    }

    return this.source.get_airports_in_range({ lat, long }, range);
  }

  public async getRunways(ident: string): Promise<RunwayThreshold[]> {
    return this.source.get_runways_at_airport(await this.requireAirport(ident));
  }

  public async getDepartures(ident: string): Promise<Departure[]> {
    return this.source.get_departures_at_airport(await this.requireAirport(ident));
  }

  public async getArrivals(ident: string): Promise<Arrival[]> {
    return this.source.get_arrivals_at_airport(await this.requireAirport(ident));
  }

  public async getApproaches(ident: string): Promise<Approach[]> {
    return this.source.get_approaches_at_airport(await this.requireAirport(ident));
  }

  public async getGates(ident: string): Promise<Gate[]> {
    return this.source.get_gates_at_airport(await this.requireAirport(ident));
  }

  public async getCommunications(ident: string): Promise<Communication[]> {
    return this.source.get_communications_at_airport(await this.requireAirport(ident));
  }

  public async getWaypoints(ident: string): Promise<Waypoint[]> {
    return this.requireFound(await this.source.get_waypoints(normalizeFixIdent(ident)), "Waypoint", ident);
  }

  public async getVhfNavaids(ident: string): Promise<VhfNavaid[]> {
    return this.requireFound(await this.source.get_vhf_navaids(normalizeFixIdent(ident)), "VHF navaid", ident);
  }

  public async getNdbNavaids(ident: string): Promise<NdbNavaid[]> {
    return this.requireFound(await this.source.get_ndb_navaids(normalizeFixIdent(ident)), "NDB navaid", ident);
  }

  public async getAirways(ident: string): Promise<Airway[]> {
    return this.requireFound(await this.source.get_airways(normalizeFixIdent(ident)), "Airway", ident);
  }

  /**
   * Normalizes an airport identifier and checks that the airport exists.
   *
   * @remarks
   * The module rejects lookups of unknown airports with a generic error, and returns empty lists for their runways, procedures, etc.,
   * so existence is checked explicitly to report a proper not found error.
   */
  private async requireAirport(ident: string): Promise<string> {
    const normalized = ident.trim().toUpperCase();
    if (!AIRPORT_IDENT.test(normalized)) {
      throw new ValidationError(`Invalid airport identifier '${ident}': expected 3-4 letters or digits`);
    }

    const rows = await this.source.execute_sql<{ ident: string }>(
      "SELECT airport_identifier AS ident FROM tbl_pa_airports WHERE airport_identifier = ?",
      [normalized],
    );
    if (rows.length === 0) {
      throw new NotFoundError(`Airport '${normalized}' not found`);
    }

    return normalized;
  }

  private requireFound<T>(items: T[], kind: string, ident: string): T[] {
    if (items.length === 0) {
      throw new NotFoundError(`${kind} '${ident.trim().toUpperCase()}' not found`);
    }
    return items;
  }
}

function normalizeFixIdent(ident: string): string {
  const normalized = ident.trim().toUpperCase();
  if (!FIX_IDENT.test(normalized)) {
    throw new ValidationError(`Invalid identifier '${ident}': expected 1-5 letters or digits`);
  }
  return normalized;
}
