import { describe, expect, test } from "bun:test";
import { NotFoundError, ValidationError } from "../src/errors";
import { MAX_RANGE_NM, NavigationDataService } from "../src/NavigationDataService";
import { expectRejection } from "./helpers";
import { CUN_WAYPOINT, MMUN, RW12L, createFakeSource } from "./fakeSource";

function createService() {
  const { source, calls } = createFakeSource();
  return { service: new NavigationDataService(source), calls };
}

describe("NavigationDataService", () => {
  test("getDatabase combines the database info and install status", async () => {
    const { service } = createService();
    const database = await service.getDatabase();
    expect(database.info.airac_cycle).toBe("2401");
    expect(database.status.installedPath).toBe("mock-navdata.sqlite");
  });

  describe("airports", () => {
    test("normalizes the identifier before looking up the airport", async () => {
      const { service, calls } = createService();
      expect(await service.getAirport(" mmun ")).toEqual(MMUN);
      expect(calls.find(c => c.method === "get_airport")?.args).toEqual(["MMUN"]);
    });

    test("rejects malformed identifiers without querying", async () => {
      const { service, calls } = createService();
      for (const ident of ["", "MM", "MMUNX", "MM-N"]) {
        await expectRejection(service.getAirport(ident), ValidationError);
      }
      expect(calls).toHaveLength(0);
    });

    test("reports unknown airports as not found", async () => {
      const { service } = createService();
      await expectRejection(service.getAirport("XXXX"), NotFoundError);
      await expectRejection(service.getRunways("XXXX"), NotFoundError);
    });

    test("returns the runways of a known airport", async () => {
      const { service } = createService();
      expect(await service.getRunways("MMUN")).toEqual([RW12L]);
    });
  });

  describe("getAirportsInRange", () => {
    test("passes valid coordinates on", async () => {
      const { service, calls } = createService();
      expect(await service.getAirportsInRange(21, -86.8, 50)).toEqual([MMUN]);
      expect(calls.at(-1)?.args).toEqual([{ lat: 21, long: -86.8 }, 50]);
    });

    test.each([
      [NaN, 0, 10],
      [91, 0, 10],
      [0, -181, 10],
      [0, 0, 0],
      [0, 0, MAX_RANGE_NM + 1],
    ])("rejects lat=%p long=%p range=%p", async (lat, long, range) => {
      const { service } = createService();
      await expectRejection(service.getAirportsInRange(lat, long, range), ValidationError);
    });
  });

  describe("enroute", () => {
    test("returns all matches", async () => {
      const { service } = createService();
      expect(await service.getWaypoints("cun")).toEqual([CUN_WAYPOINT]);
    });

    test("reports no match as not found", async () => {
      const { service } = createService();
      await expectRejection(service.getWaypoints("NOPE"), NotFoundError);
      await expectRejection(service.getAirways("UJ3"), NotFoundError);
    });

    test("rejects malformed identifiers", async () => {
      const { service } = createService();
      await expectRejection(service.getVhfNavaids("TOOLONG"), ValidationError);
    });
  });
});
