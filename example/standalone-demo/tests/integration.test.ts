import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { NotFoundError } from "../src/errors";
import { DEFAULT_WASM_PATH, type LoadedNavigationData, loadNavigationData } from "../src/loadNavigationData";
import { NavigationDataService } from "../src/NavigationDataService";
import { expectRejection, readJson, startApp } from "./helpers";

interface OpenApiSpec {
  paths: Record<string, { get: { parameters?: { name: string; in: string; schema: { example?: unknown } }[] } }>;
}

/// Runs against the real standalone module, so it is skipped until `bun run build:wasm:standalone` has been run
describe.skipIf(!existsSync(DEFAULT_WASM_PATH))("NavigationDataService with the standalone module", () => {
  let loaded: LoadedNavigationData;
  let service: NavigationDataService;

  beforeAll(async () => {
    loaded = await loadNavigationData();
    service = new NavigationDataService(loaded.navigationDataInterface);
  });

  afterAll(() => {
    loaded?.transport.dispose();
  });

  test("serves the mock database", async () => {
    const { info, status } = await service.getDatabase();
    expect(info.airac_cycle).toMatch(/^\d{4}$/);
    expect(status.installedPath).toBe("mock-navdata.sqlite");
  });

  test("serves a mock airport with its runways and procedures", async () => {
    const airport = await service.getAirport("mmun");
    expect(airport.ident).toBe("MMUN");
    expect((await service.getRunways("MMUN")).length).toBeGreaterThan(0);
    expect((await service.getApproaches("MMUN")).length).toBeGreaterThan(0);
  });

  test("finds the airport by range", async () => {
    const airport = await service.getAirport("MMUN");
    const nearby = await service.getAirportsInRange(airport.location.lat, airport.location.long, 10);
    expect(nearby.map(a => a.ident)).toContain("MMUN");
  });

  test("reports airports outside the mock data as not found", async () => {
    await expectRejection(service.getAirport("EGLL"), NotFoundError);
  });

  test("every example in the OpenAPI spec resolves", async () => {
    const { app, baseUrl } = await startApp(loaded.navigationDataInterface);

    try {
      const spec = await readJson<OpenApiSpec>(await fetch(`${baseUrl}/openapi.json`));
      for (const [path, item] of Object.entries(spec.paths)) {
        // Fill each parameter with its documented example
        let url = path;
        const query = new URLSearchParams();
        for (const p of item.get.parameters ?? []) {
          if (p.in === "path") url = url.replace(`{${p.name}}`, String(p.schema.example));
          else query.set(p.name, String(p.schema.example));
        }

        const res = await fetch(`${baseUrl}${url}${query.size > 0 ? `?${query}` : ""}`);
        expect({ path, status: res.status }).toEqual({ path, status: 200 });
      }
    } finally {
      await app.close();
    }
  });
});
