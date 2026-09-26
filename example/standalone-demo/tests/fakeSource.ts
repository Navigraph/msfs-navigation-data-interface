import type { Airport, RunwayThreshold, Waypoint } from "../src/navigraph";
import type { NavigationDataSource } from "../src/NavigationDataService";

export const MMUN = {
  ident: "MMUN",
  icao_code: "MM",
  area_code: "SAM",
  name: "CANCUN INTL",
  location: { lat: 21.04, long: -86.87 },
  elevation: 22,
} as Airport;

export const RW12L = { ident: "RW12L", length: 11483 } as RunwayThreshold;

export const CUN_WAYPOINT = { ident: "CUN", location: { lat: 21.03, long: -86.86 } } as Waypoint;

/**
 * An in-memory stand-in for the JS interface, holding one airport (MMUN) and one waypoint (CUN).
 * Records the arguments of each call, so tests can check what the service passed on.
 */
export function createFakeSource() {
  const calls: { method: string; args: unknown[] }[] = [];
  const record =
    <T>(method: string, result: T) =>
    (...args: unknown[]) => {
      calls.push({ method, args });
      return Promise.resolve(result);
    };

  const source: NavigationDataSource = {
    execute_sql: <T>(sql: string, params: string[]) => {
      calls.push({ method: "execute_sql", args: [sql, params] });
      return Promise.resolve((params[0] === MMUN.ident ? [{ ident: MMUN.ident }] : []) as T[]);
    },
    get_database_info: record("get_database_info", {
      airac_cycle: "2401",
      effective_from_to: ["25-01-2024", "21-02-2024"] as [string, string],
      previous_from_to: ["28-12-2023", "25-01-2024"] as [string, string],
    }),
    get_navigation_data_install_status: record("get_navigation_data_install_status", {
      status: "Manual",
      installedPath: "mock-navdata.sqlite",
    } as Awaited<ReturnType<NavigationDataSource["get_navigation_data_install_status"]>>),
    get_airport: record("get_airport", MMUN),
    get_airports_in_range: record("get_airports_in_range", [MMUN]),
    get_runways_at_airport: record("get_runways_at_airport", [RW12L]),
    get_departures_at_airport: record("get_departures_at_airport", []),
    get_arrivals_at_airport: record("get_arrivals_at_airport", []),
    get_approaches_at_airport: record("get_approaches_at_airport", []),
    get_gates_at_airport: record("get_gates_at_airport", []),
    get_communications_at_airport: record("get_communications_at_airport", []),
    get_waypoints: (ident: string) => record("get_waypoints", ident === "CUN" ? [CUN_WAYPOINT] : [])(ident),
    get_vhf_navaids: record("get_vhf_navaids", []),
    get_ndb_navaids: record("get_ndb_navaids", []),
    get_airways: record("get_airways", []),
  };

  return { source, calls };
}
