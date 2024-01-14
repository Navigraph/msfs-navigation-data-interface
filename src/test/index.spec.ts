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
import { NdbNavaid } from "../js/types/ndb_navaid"
import { AltitudeDescriptor, LegType } from "../js/types/ProcedureLeg"
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
    })
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
})
