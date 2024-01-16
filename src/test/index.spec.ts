import {
  Airport,
  AirwayLevel,
  AirwayRouteType,
  Fix,
  FixType,
  IfrCapability,
  NavigraphNavdataInterface,
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
import { RunwayThreshold } from "../js/types/runway_threshold"
import { VhfNavaid } from "../js/types/vhfnavaid"
import { Waypoint } from "../js/types/waypoint"

const navdataInterface = new NavigraphNavdataInterface()

describe("test", () => {
  it("Database info", async () => {
    const info = await navdataInterface.get_database_info("KJFK")

    expect(info).toStrictEqual({
      airac_cycle: "2313",
      effective_from_to: ["28-12-2023", "25-01-2024"],
      previous_from_to: ["30-11-2023", "28-12-2023"],
    } satisfies DatabaseInfo)
  })

  it("Fetch airport", async () => {
    const airport = await navdataInterface.get_airport("KJFK")

    expect(airport).toStrictEqual({
      area_code: "USA",
      ident: "KJFK",
      icao_code: "K6",
      location: {
        lat: 40.63992778,
        long: -73.77869167,
      },
      name: "KENNEDY INTL",
      ifr_capability: IfrCapability.Yes,
      longest_runway_surface_code: RunwaySurfaceCode.Hard,
      elevation: 13,
      transition_altitude: 18000,
      transition_level: 18000,
      speed_limit: 250,
      speed_limit_altitude: 10000,
      iata_ident: "JFK",
    } satisfies Airport)
  })

  it("Get waypoints", async () => {
    const waypoints = await navdataInterface.get_waypoints("GLENN")

    expect(waypoints.length).toBe(3)

    expect(waypoints[0]).toStrictEqual({
      area_code: "SPA",
      icao_code: "NZ",
      ident: "GLENN",
      location: {
        lat: -42.88116389,
        long: 172.83973889,
      },
      name: "GLENN",
    } satisfies Waypoint)
  })

  it("Get vhf navaids", async () => {
    const navaids = await navdataInterface.get_vhf_navaids("CH")

    expect(navaids.length).toBe(3)

    expect(navaids[0]).toStrictEqual({
      area_code: "EUR",
      icao_code: "EK",
      ident: "CH",
      location: {
        lat: 55.59326389,
        long: 12.60829167,
      },
      frequency: 110.5,
      name: "KASTRUP",
    } satisfies VhfNavaid)
  })

  it("Get ndb navaids", async () => {
    const navaids = await navdataInterface.get_ndb_navaids("CH")

    expect(navaids.length).toBe(4)

    expect(navaids[0]).toStrictEqual({
      area_code: "AFR",
      icao_code: "FQ",
      ident: "CH",
      location: {
        lat: -19.10385,
        long: 33.43294722,
      },
      frequency: 282,
      name: "CHIMOIO",
    } satisfies NdbNavaid)
  })

  it("Get airports in range", async () => {
    const airports = await navdataInterface.get_airports_in_range({ lat: 51.468, long: -0.4551 }, 640)

    expect(airports.length).toBe(1686)
  })

  it("Get waypoints in range", async () => {
    const waypoints = await navdataInterface.get_waypoints_in_range({ lat: -43.4876, long: 172.5374 }, 10)

    expect(waypoints.length).toBe(126)
  })

  it("Get vhf navaids in range", async () => {
    const vhf_navaids = await navdataInterface.get_vhf_navaids_in_range({ lat: -43.4876, long: 172.5374 }, 10)

    expect(vhf_navaids.length).toBe(3)
  })

  it("Get ndb navaids in range", async () => {
    const ndb_navaids = await navdataInterface.get_ndb_navaids_in_range({ lat: -45.9282, long: 170.1981 }, 5)

    expect(ndb_navaids.length).toBe(1)
  })

  it("Get controlled airspaces in range", async () => {
    const airspaces = await navdataInterface.get_controlled_airspaces_in_range({ lat: -43.4876, long: 172.5374 }, 10)

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
        lat: -39.03916667,
        long: 173.54138889,
      },
      path_type: PathType.GreatCircle,
    } satisfies Path)

    expect(target_airspace.boundary_paths[1]).toStrictEqual({
      location: {
        lat: -40.77753611,
        long: 172.74154167,
      },
      arc: {
        bearing: 288.9,
        direction: TurnDirection.Left,
        distance: 100,
        origin: {
          lat: -41.33722778,
          long: 174.81696111,
        },
      },
      path_type: PathType.Arc,
    } satisfies Path)
  })

  it("Get restrictive airspaces in range", async () => {
    const airspaces = await navdataInterface.get_restrictive_airspaces_in_range({ lat: -43.4876, long: 172.5374 }, 10)

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
        lat: -43.46666667,
        long: 172.36977778,
      },
      path_type: PathType.GreatCircle,
    } satisfies Path)
  })

  it("Get communications in range", async () => {
    const communications = await navdataInterface.get_communications_in_range({ lat: -43.4876, long: 172.5374 }, 10)

    expect(communications.length).toBe(46)
  })

  it("Get airways", async () => {
    const airways = await navdataInterface.get_airways("A1")

    const target_airway = airways[0]

    expect(airways.length).toBe(5)
    expect(airways[0].direction).toBeUndefined()
    expect(target_airway.fixes.length).toBe(52)
    expect(target_airway.ident).toBe("A1")
    expect(target_airway.level).toBe(AirwayLevel.Both)
    expect(target_airway.route_type).toBe(AirwayRouteType.OfficialDesignatedAirwaysExpectRnavAirways)
    expect(target_airway.fixes[0]).toStrictEqual({
      fix_type: FixType.VhfNavaid,
      ident: "KEC",
      icao_code: "RJ",
      location: {
        lat: 33.44774167,
        long: 135.79449444,
      },
    } satisfies Fix)
  })

  it("Get airways at fix", async () => {
    const airways = await navdataInterface.get_airways_at_fix("ODOWD", "NZ")

    expect(airways.length).toBe(4)
  })

  it("Get airways in range", async () => {
    const airways = await navdataInterface.get_airways_in_range({ lat: -43.4876, long: 172.5374 }, 10)

    expect(airways.length).toBe(27)
  })

  it("Get runways at airport", async () => {
    const runways = await navdataInterface.get_runways_at_airport("NZCH")

    expect(runways.length).toBe(4)

    const target_runway = runways[0]

    expect(target_runway).toStrictEqual({
      icao_code: "NZ",
      ident: "RW02",
      elevation: 123,
      gradient: -0.278,
      length: 10787,
      width: 148,
      location: {
        lat: -43.49763056,
        long: 172.52211389,
      },
      magnetic_bearing: 16,
      true_bearing: 39.995,
    } satisfies RunwayThreshold)
  })

  it("Get departures", async () => {
    const departures = await navdataInterface.get_departures_at_airport("KLAX")

    expect(departures.length).toBe(22)

    const target_departure = departures.find(departure => departure.ident === "PNDAH2")

    expect(target_departure.ident).toBe("PNDAH2")
    expect(target_departure.runway_transitions.length).toBe(4)
    expect(target_departure.enroute_transitions.length).toBe(2)
    expect(target_departure.common_legs.length).toBe(4)
    expect(target_departure.runway_transitions[0].ident).toBe("RW24L")
    expect(target_departure.runway_transitions[0].legs.length).toBe(6)
    expect(target_departure.enroute_transitions[0].ident).toBe("OTAYY")
    expect(target_departure.enroute_transitions[0].legs.length).toBe(2)
  })

  it("Get Arrivals", async () => {
    const arrivals = await navdataInterface.get_arrivals_at_airport("KLAX")

    expect(arrivals.length).toBe(24)

    const target_arrival = arrivals.find(arrival => arrival.ident === "BRUEN2")

    expect(target_arrival.ident).toBe("BRUEN2")
    expect(target_arrival.enroute_transitions.length).toBe(4)
    expect(target_arrival.runway_transitions.length).toBe(4)
    expect(target_arrival.common_legs.length).toBe(7)
    expect(target_arrival.enroute_transitions[0].ident).toBe("ESTWD")
    expect(target_arrival.enroute_transitions[0].legs.length).toBe(5)
    expect(target_arrival.runway_transitions[0].ident).toBe("RW06L")
    expect(target_arrival.runway_transitions[0].legs.length).toBe(8)
  })

  it("Get Approaches", async () => {
    const approaches = await navdataInterface.get_approaches_at_airport("KLAX")

    expect(approaches.length).toBe(24)

    const target_approach = approaches.find(approach => approach.ident === "I06L")

    expect(target_approach.ident).toBe("I06L")
    expect(target_approach.legs.length).toBe(3)
    expect(target_approach.missed_legs.length).toBe(3)
    expect(target_approach.runway_ident).toBe("RW06L")
    expect(target_approach.approach_type).toBe(ApproachType.Ils)
    expect(target_approach.transitions.length).toBe(3)
    expect(target_approach.transitions[0].ident).toBe("CLVVR")
    expect(target_approach.transitions[0].legs.length).toBe(2)
  })

  it("Get waypoints at airport", async () => {
    const waypoints = await navdataInterface.get_waypoints_at_airport("NZCH")

    expect(waypoints.length).toBe(200)
  })

  it("Get ndb navaids at airport", async () => {
    const navaids = await navdataInterface.get_ndb_navaids_at_airport("EDDM")

    expect(navaids.length).toBe(4)
  })

  it("Check procedure leg types", async () => {
    // This airport has the most different leg types
    const approaches = await navdataInterface.get_approaches_at_airport("GCLP")

    const approach1 = approaches.find(approach => approach.ident == "L21RZ")

    const IF = approach1.legs[0]

    expect(IF).toStrictEqual<IFLegData>({
      leg_type: LegType.IF,
      overfly: false,
      fix: {
        airport_ident: "GCLP",
        fix_type: FixType.Waypoint,
        ident: "CF21R",
        icao_code: "GC",
        location: {
          lat: 28.116,
          long: -15.30502778,
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
          lat: 27.91594444,
          long: -15.39363889,
        },
      },
    })
  })

  it("Get gates at airport", async () => {
    const gates = await navdataInterface.get_gates_at_airport("NZCH")

    expect(gates.length).toBe(48)

    expect(gates[0]).toStrictEqual({
      area_code: "SPA",
      icao_code: "NZ",
      ident: "10",
      location: {
        lat: -43.49016944,
        long: 172.53940833,
      },
      name: "10",
    } satisfies Gate)
  })

  it("Get communications at airport", async () => {
    const communications = await navdataInterface.get_communications_at_airport("NZCH")

    expect(communications.length).toBe(14)

    expect(communications[0]).toStrictEqual({
      area_code: "SPA",
      airport_ident: "NZCH",
      communication_type: CommunicationType.ApproachControl,
      frequency: 120.9,
      frequency_units: FrequencyUnits.VeryHigh,
      callsign: "CHRISTCHURCH",
      location: {
        lat: -43.48944444,
        long: 172.53444444,
      },
    } satisfies Communication)
  })

  it("Get GlsNavaids at airport", async () => {
    const communications = await navdataInterface.get_gls_navaids_at_airport("YSSY")

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
        lat: -33.96333333,
        long: 151.18477778,
      },
      magnetic_approach_bearing: 62,
      magnetic_variation: 13,
    } satisfies GlsNavaid)
  })

  it("Get PathPoints at airport", async () => {
    const pathpoints = await navdataInterface.get_path_points_at_airport("KLAX")

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
        lat: 33.95213333,
        long: -118.40162778,
      },
      fpap_ellipsoid_height: -1.5,
      glidepath_angle: 3,
      gnss_channel_number: 82507,
      horizontal_alert_limit: 40,
      vertical_alert_limit: 50,
      landing_threshold_location: {
        lat: 33.94911111,
        long: -118.43115833,
      },
      length_offset: 32,
      ltp_ellipsoid_height: -1.5,
      path_point_tch: 16.671746418774763,
      fpap_orthometric_height: 34.5,
      ltp_orthometric_height: 34.5,
    } satisfies PathPoint)
  })
})
