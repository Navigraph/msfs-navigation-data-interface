import {
  Airport,
  AirwayLevel,
  AirwayRouteType,
  Fix,
  FixType,
  IfrCapability,
  NavigraphNavigationDataInterface,
  RunwaySurfaceCode,
} from "../js"
import { ControlledAirspaceType, Path, PathType, RestrictiveAirspaceType } from "../js/types/airspace"
import { Communication, CommunicationType, FrequencyUnits } from "../js/types/communication"
import { DatabaseInfo } from "../js/types/database_info"
import { Gate } from "../js/types/gate"
import { GlsNavaid } from "../js/types/gls_navaid"
import { NdbNavaid } from "../js/types/ndb_navaid"
import { ApproachTypeIdentifier, PathPoint } from "../js/types/path_point"
import { ApproachType } from "../js/types/procedure"
import { AltitudeDescriptor, LegType, TurnDirection } from "../js/types/ProcedureLeg"
import { IFLegData } from "../js/types/ProcedureLeg/IFLeg"
import { RunwayLights, RunwaySurface, RunwayThreshold, TrafficPattern } from "../js/types/runway_threshold"
import { VhfNavaid } from "../js/types/vhfnavaid"
import { Waypoint } from "../js/types/waypoint"

const navigationDataInterface = new NavigraphNavigationDataInterface()

describe("test", () => {
  it("Database info", async () => {
    const info = await navigationDataInterface.get_database_info("KJFK")

    expect(info).toStrictEqual({
      airac_cycle: "2410",
      effective_from_to: ["03-10-2024", "30-10-2024"],
      previous_from_to: ["depricated", "depricated"],
    } satisfies DatabaseInfo)
  })

  it("Fetch airport", async () => {
    const airport = await navigationDataInterface.get_airport("KJFK")

    expect(airport).toStrictEqual({
      airport_type: "C",
      area_code: "USA",
      city: "NEW YORK",
      continent: "NORTH AMERICA",
      country: "UNITED STATES",
      country_3letter: "USA",
      ident: "KJFK",
      icao_code: "K6",
      location: {
        lat: 40.63992777777778,
        long: -73.77869166666666,
      },
      name: "KENNEDY INTL",
      ifr_capability: IfrCapability.Yes,
      longest_runway_surface_code: RunwaySurfaceCode.Hard,
      magnetic_variation: -13,
      elevation: 13,
      transition_altitude: 18000,
      transition_level: 18000,
      speed_limit: 250,
      speed_limit_altitude: 10000,
      state: "NEW YORK",
      state_2letter: "NY",
      iata_ident: "JFK",
    } satisfies Airport)
  })

  it("Get waypoints", async () => {
    const waypoints = await navigationDataInterface.get_waypoints("GLENN")

    expect(waypoints.length).toBe(3)

    expect(waypoints[0]).toStrictEqual({
      area_code: "SPA",
      continent: "PACIFIC",
      country: "NEW ZEALAND",
      datum_code: "WGE",
      icao_code: "NZ",
      ident: "GLENN",
      location: {
        lat: -42.88116388888889,
        long: 172.8397388888889,
      },
      name: "GLENN",
    } satisfies Waypoint)
  })

  it("Get vhf navaids", async () => {
    const navaids = await navigationDataInterface.get_vhf_navaids("CH")

    expect(navaids.length).toBe(3)

    expect(navaids[0]).toStrictEqual({
      airport_ident: "EKCH",
      area_code: "EUR",
      continent: "EUROPE",
      country: "DENMARK",
      datum_code: "WGE",
      icao_code: "EK",
      ident: "CH",
      location: {
        lat: 55.59326388888889,
        long: 12.608291666666666,
      },
      frequency: 110.5,
      name: "KASTRUP",
      magnetic_variation: 5.1,
      range: 25,
    } satisfies VhfNavaid)
  })

  it("Get ndb navaids", async () => {
    const navaids = await navigationDataInterface.get_ndb_navaids("CH")

    expect(navaids.length).toBe(4)

    expect(navaids[0]).toStrictEqual({
      area_code: "AFR",
      continent: "AFRICA",
      country: "MOZAMBIQUE",
      datum_code: "WGE",
      icao_code: "FQ",
      ident: "CH",
      location: {
        lat: -19.10385,
        long: 33.432947222222225,
      },
      frequency: 282,
      name: "CHIMOIO",
      range: 75,
    } satisfies NdbNavaid)
  })

  it("Get airports in range", async () => {
    const airports = await navigationDataInterface.get_airports_in_range({ lat: 51.468, long: -0.4551 }, 640)

    expect(airports.length).toBe(1506)
  })

  it("Get waypoints in range", async () => {
    const waypoints = await navigationDataInterface.get_waypoints_in_range({ lat: -43.4876, long: 172.5374 }, 10)

    expect(waypoints.length).toBe(126)
  })

  it("Get vhf navaids in range", async () => {
    const vhf_navaids = await navigationDataInterface.get_vhf_navaids_in_range({ lat: -43.4876, long: 172.5374 }, 10)

    expect(vhf_navaids.length).toBe(1)
  })

  it("Get ndb navaids in range", async () => {
    const ndb_navaids = await navigationDataInterface.get_ndb_navaids_in_range({ lat: -45.9282, long: 170.1981 }, 5)

    expect(ndb_navaids.length).toBe(1)
  })

  it("Get controlled airspaces in range", async () => {
    const airspaces = await navigationDataInterface.get_controlled_airspaces_in_range(
      { lat: -43.4876, long: 172.5374 },
      10,
    )

    expect(airspaces.length).toBe(17)

    const target_airspace = airspaces[1]

    expect(target_airspace.airspace_center).toBe("NZCH")
    expect(target_airspace.airspace_type).toBe(ControlledAirspaceType.TmaOrTca)
    expect(target_airspace.area_code).toBe("SPA")
    expect(target_airspace.icao_code).toBe("NZ")
    expect(target_airspace.name).toBe("CHRISTCHURCH CTA/C")
    expect(target_airspace.boundary_paths.length).toBe(11)

    expect(target_airspace.boundary_paths[0]).toStrictEqual({
      location: {
        lat: -39.03916666666667,
        long: 173.5413888888889,
      },
      path_type: PathType.GreatCircle,
    } satisfies Path)

    expect(target_airspace.boundary_paths[1]).toStrictEqual({
      location: {
        lat: -40.77753611111111,
        long: 172.74154166666668,
      },
      arc: {
        bearing: 288.9,
        direction: TurnDirection.Left,
        distance: 100,
        origin: {
          lat: -41.33722777777778,
          long: 174.8169611111111,
        },
      },
      path_type: PathType.Arc,
    } satisfies Path)
  })

  it("Get restrictive airspaces in range", async () => {
    const airspaces = await navigationDataInterface.get_restrictive_airspaces_in_range(
      { lat: -43.4876, long: 172.5374 },
      10,
    )

    expect(airspaces.length).toBe(5)

    const target_airspace = airspaces[0]

    expect(target_airspace.area_code).toBe("SPA")
    expect(target_airspace.icao_code).toBe("NZ")
    expect(target_airspace.name).toBe("WEST MELTON, CANTERBURY")
    expect(target_airspace.airspace_type).toBe(RestrictiveAirspaceType.Danger)
    expect(target_airspace.designation).toBe("827")
    expect(target_airspace.boundary_paths.length).toBe(8)
    expect(target_airspace.boundary_paths[0]).toStrictEqual({
      location: {
        lat: -43.46666666666667,
        long: 172.36977777777778,
      },
      path_type: PathType.GreatCircle,
    } satisfies Path)
  })

  it("Get communications in range", async () => {
    const communications = await navigationDataInterface.get_communications_in_range(
      { lat: -43.4876, long: 172.5374 },
      10,
    )

    expect(communications.length).toBe(48)
  })

  it("Get airways", async () => {
    const airways = await navigationDataInterface.get_airways("A1")

    const target_airway = airways[1]

    expect(airways.length).toBe(3)
    expect(airways[0].direction).toBeUndefined()
    expect(target_airway.fixes.length).toBe(36)
    expect(target_airway.ident).toBe("A1")
    expect(target_airway.level).toBe(AirwayLevel.Both)
    expect(target_airway.route_type).toBe(AirwayRouteType.OfficialDesignatedAirwaysExpectRnavAirways)
    expect(target_airway.fixes[0]).toStrictEqual({
      fix_type: FixType.VhfNavaid,
      ident: "KEC",
      icao_code: "RJ",
      location: {
        lat: 33.447741666666666,
        long: 135.79449444444444,
      },
    } satisfies Fix)
  })

  it("Get airways at fix", async () => {
    const airways = await navigationDataInterface.get_airways_at_fix("ODOWD", "NZ")

    expect(airways.length).toBe(4)
  })

  it("Get airways in range", async () => {
    const airways = await navigationDataInterface.get_airways_in_range({ lat: -43.4876, long: 172.5374 }, 10)

    expect(airways.length).toBe(27)
  })

  it("Get runways at airport", async () => {
    const runways = await navigationDataInterface.get_runways_at_airport("NZCH")

    expect(runways.length).toBe(4)

    const target_runway = runways[0]

    expect(target_runway).toStrictEqual({
      icao_code: "NZ",
      ident: "RW02",
      elevation: 123,
      gradient: -0.28,
      length: 10787,
      width: 148,
      lights: RunwayLights.Yes,
      location: {
        lat: -43.49763055555555,
        long: 172.5221138888889,
      },
      magnetic_bearing: 16,
      true_bearing: 40.0,
      surface: RunwaySurface.Bitumen,
      traffic_pattern: TrafficPattern.Left,
    } satisfies RunwayThreshold)
  })

  it("Get departures", async () => {
    const departures = await navigationDataInterface.get_departures_at_airport("KLAX")

    expect(departures.length).toBe(24)

    const target_departure = departures.find(departure => departure.ident === "PNDAH2")

    expect(target_departure?.ident).toBe("PNDAH2")
    expect(target_departure?.runway_transitions.length).toBe(4)
    expect(target_departure?.enroute_transitions.length).toBe(2)
    expect(target_departure?.common_legs.length).toBe(4)
    expect(target_departure?.runway_transitions[0].ident).toBe("RW24L")
    expect(target_departure?.runway_transitions[0].legs.length).toBe(6)
    expect(target_departure?.enroute_transitions[0].ident).toBe("OTAYY")
    expect(target_departure?.enroute_transitions[0].legs.length).toBe(2)
  })

  it("Get Arrivals", async () => {
    const arrivals = await navigationDataInterface.get_arrivals_at_airport("KLAX")

    expect(arrivals.length).toBe(24)

    const target_arrival = arrivals.find(arrival => arrival.ident === "BRUEN2")

    expect(target_arrival?.ident).toBe("BRUEN2")
    expect(target_arrival?.enroute_transitions.length).toBe(4)
    expect(target_arrival?.runway_transitions.length).toBe(4)
    expect(target_arrival?.common_legs.length).toBe(7)
    expect(target_arrival?.enroute_transitions[0].ident).toBe("ESTWD")
    expect(target_arrival?.enroute_transitions[0].legs.length).toBe(5)
    expect(target_arrival?.runway_transitions[0].ident).toBe("RW06L")
    expect(target_arrival?.runway_transitions[0].legs.length).toBe(8)
  })

  it("Get Approaches", async () => {
    const approaches = await navigationDataInterface.get_approaches_at_airport("KLAX")

    expect(approaches.length).toBe(24)

    const target_approach = approaches.find(approach => approach.ident === "I06L")

    expect(target_approach?.ident).toBe("I06L")
    expect(target_approach?.legs.length).toBe(3)
    expect(target_approach?.missed_legs.length).toBe(3)
    expect(target_approach?.runway_ident).toBe("RW06L")
    expect(target_approach?.approach_type).toBe(ApproachType.Ils)
    expect(target_approach?.transitions.length).toBe(3)
    expect(target_approach?.transitions[0].ident).toBe("CLVVR")
    expect(target_approach?.transitions[0].legs.length).toBe(2)
  })

  it("Get waypoints at airport", async () => {
    const waypoints = await navigationDataInterface.get_waypoints_at_airport("NZCH")

    expect(waypoints.length).toBe(201)
  })

  it("Get ndb navaids at airport", async () => {
    const navaids = await navigationDataInterface.get_ndb_navaids_at_airport("EDDM")

    expect(navaids.length).toBe(4)
  })

  it("Check procedure leg types", async () => {
    // This airport has the most different leg types
    const approaches = await navigationDataInterface.get_approaches_at_airport("GCLP")

    const approach1 = approaches.find(approach => approach.ident == "L21RZ")

    const IF = approach1?.legs[0]

    expect(IF).toStrictEqual<IFLegData>({
      leg_type: LegType.IF,
      overfly: false,
      fix: {
        airport_ident: "GCLP",
        fix_type: FixType.Waypoint,
        ident: "TIPUX",
        icao_code: "GC",
        location: {
          lat: 28.116,
          long: -15.305055555555555,
        },
      },
      theta: 25.4,
      rho: 12.9,
      altitude: {
        altitude1: 2500,
        descriptor: AltitudeDescriptor.AtOrAboveAlt1,
      },
      recommended_navaid: {
        airport_ident: "GCLP",
        fix_type: FixType.IlsNavaid,
        ident: "RLP",
        icao_code: "GC",
        location: {
          lat: 27.915944444444445,
          long: -15.393638888888889,
        },
      },
    })
  })

  it("Get gates at airport", async () => {
    const gates = await navigationDataInterface.get_gates_at_airport("NZCH")

    expect(gates.length).toBe(48)

    expect(gates[0]).toStrictEqual({
      area_code: "SPA",
      icao_code: "NZ",
      ident: "10",
      location: {
        lat: -43.49016944444445,
        long: 172.53940833333334,
      },
      name: "N/A",
    } satisfies Gate)
  })

  it("Get communications at airport", async () => {
    const communications = await navigationDataInterface.get_communications_at_airport("NZCH")

    expect(communications.length).toBe(17)

    expect(communications[3]).toStrictEqual({
      area_code: "SPA",
      airport_ident: "NZCH",
      communication_type: CommunicationType.ApproachControl,
      frequency: 126.1,
      frequency_units: FrequencyUnits.VeryHigh,
      callsign: "CHRISTCHURCH",
      location: {
        lat: -43.489444444444445,
        long: 172.53444444444443,
      },
    } satisfies Communication)
  })

  it("Get GlsNavaids at airport", async () => {
    const communications = await navigationDataInterface.get_gls_navaids_at_airport("YSSY")

    expect(communications.length).toBe(6)

    expect(communications[0]).toStrictEqual({
      area_code: "SPA",
      airport_ident: "YSSY",
      icao_code: "YM",
      ident: "G07A",
      category: "1",
      channel: 22790,
      runway_ident: "RW07",
      approach_angle: 3,
      elevation: 21,
      location: {
        lat: -33.96333333333333,
        long: 151.18477777777778,
      },
      magnetic_approach_bearing: 62,
      magnetic_variation: 13,
    } satisfies GlsNavaid)
  })

  it("Get PathPoints at airport", async () => {
    const pathpoints = await navigationDataInterface.get_path_points_at_airport("KLAX")

    expect(pathpoints.length).toBe(8)

    expect(pathpoints[0]).toStrictEqual({
      area_code: "USA",
      airport_ident: "KLAX",
      icao_code: "K2",
      ident: "W06A",
      runway_ident: "RW06L",
      approach_ident: "R06LY",
      approach_type: ApproachTypeIdentifier.LocalizerPerformanceVerticalGuidance,
      course_width: 106.75,
      flightpath_alignment_location: {
        lat: 33.952133333333336,
        long: -118.40162777777778,
      },
      glidepath_angle: 3,
      gnss_channel_number: 82507,
      horizontal_alert_limit: 40,
      vertical_alert_limit: 50,
      landing_threshold_location: {
        lat: 33.94911111111111,
        long: -118.43115833333333,
      },
      length_offset: 32,
      ltp_ellipsoid_height: -1.5,
      path_point_tch: 16.6725594664781,
    } satisfies PathPoint)
  })
})
