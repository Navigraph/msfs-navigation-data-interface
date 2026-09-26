import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import type { INestApplication } from "@nestjs/common";
import type { DatabaseSummary } from "../src/NavigationDataService";
import { MMUN, createFakeSource } from "./fakeSource";
import { type ErrorResponse, readJson, startApp } from "./helpers";

let app: INestApplication;
let baseUrl: string;

beforeAll(async () => {
  ({ app, baseUrl } = await startApp(createFakeSource().source));
});

afterAll(async () => {
  await app.close();
});

const get = (path: string) => fetch(baseUrl + path);

describe("HTTP API", () => {
  test("GET /health", async () => {
    const res = await get("/health");
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ status: "ok" });
  });

  test("GET /api/database", async () => {
    const res = await get("/api/database");
    expect(res.status).toBe(200);
    expect((await readJson<DatabaseSummary>(res)).info.airac_cycle).toBe("2401");
  });

  test("GET /api/airports/:ident", async () => {
    const res = await get("/api/airports/mmun");
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual(MMUN);
  });

  test("GET /api/airports/:ident/runways", async () => {
    const res = await get("/api/airports/MMUN/runways");
    expect(res.status).toBe(200);
    expect(await res.json()).toHaveLength(1);
  });

  test("GET /api/airports with query parameters", async () => {
    const res = await get("/api/airports?lat=21&long=-86.8&range=50");
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual([MMUN]);
  });

  test("responds 400 to invalid input", async () => {
    for (const path of [
      "/api/airports/M!",
      "/api/airports?lat=21&long=-86.8",
      "/api/airports?lat=abc&long=0&range=5",
      "/api/airports?lat=95&long=0&range=5",
    ]) {
      const res = await get(path);
      expect({ path, status: res.status }).toEqual({ path, status: 400 });
    }
  });

  test("responds 404 when nothing matches", async () => {
    const res = await get("/api/airports/XXXX/approaches");
    expect(res.status).toBe(404);
    expect(await res.json()).toEqual({ statusCode: 404, error: "Not Found", message: "Airport 'XXXX' not found" });
  });

  test("responds 500 to unexpected errors", async () => {
    const { source } = createFakeSource();
    source.get_database_info = () => Promise.reject(new Error("module failure"));
    const failing = await startApp(source);
    try {
      const res = await fetch(`${failing.baseUrl}/api/database`);
      expect(res.status).toBe(500);
      expect((await readJson<ErrorResponse>(res)).statusCode).toBe(500);
    } finally {
      await failing.app.close();
    }
  });

  test("serves the OpenAPI spec and Swagger UI", async () => {
    const spec = await readJson<{ openapi: string; paths: Record<string, unknown> }>(await get("/openapi.json"));
    expect(spec.openapi).toStartWith("3.");
    expect(Object.keys(spec.paths)).toContain("/api/airports/{ident}/runways");

    const docs = await get("/docs");
    expect(docs.status).toBe(200);
    expect(await docs.text()).toContain("swagger-ui");
  });
});
