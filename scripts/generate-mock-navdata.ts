import { Database } from "bun:sqlite";
import { copyFileSync, existsSync, mkdirSync, rmSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

/// Generates the mock navigation data served by the standalone WASM build (`mock-navdata.sqlite` and `mock-cycle.json`).
///
/// The data is a subset of the example bundled database, so the schema is identical to real navigation data and every
/// query of the interface works against it.
///
/// This runs inside the standalone build container (see scripts/cargo-standalone.ts), which embeds the output into the module.
/// Usage: `bun ./scripts/generate-mock-navdata.ts --out <dir> [AIRPORT ...]`

const workspaceRoot = resolve(import.meta.dir, "..");
const sourceDir = join(workspaceRoot, "example/aircraft/PackageSources/Navigraph/BundledData");
const sourceDb = join(sourceDir, "db.s3db");

/// Airports to include, with all their procedures, runways, gates, etc. Everything else is limited to the area around them.
const DEFAULT_AIRPORTS = ["MMUN", "MMMD", "MSLP"];

/// Size of the area (in degrees, in each direction) around each airport in which enroute data is kept
const AREA_MARGIN_DEGREES = 2;

const args = process.argv.slice(2);
const outIndex = args.indexOf("--out");
const outArg = outIndex >= 0 ? args[outIndex + 1] : undefined;
if (!outArg) {
  console.error("[-] Usage: generate-mock-navdata.ts --out <dir> [AIRPORT ...]");
  process.exit(1);
}
const outDir = resolve(workspaceRoot, outArg);
const outDb = join(outDir, "mock-navdata.sqlite");
const outCycle = join(outDir, "mock-cycle.json");

const airportArgs = args.filter((_, i) => i !== outIndex && i !== outIndex + 1).map(a => a.toUpperCase());
const airports = airportArgs.length > 0 ? airportArgs : DEFAULT_AIRPORTS;

if (!existsSync(sourceDb)) {
  console.error(`[-] Source database not found at ${sourceDb}`);
  process.exit(1);
}

mkdirSync(outDir, { recursive: true });
rmSync(outDb, { force: true });

const db = new Database(outDb, { create: true });
db.run(`ATTACH DATABASE '${sourceDb.replaceAll("'", "''")}' AS src`);

// Copy the schema (tables first, then indexes and views)
const schema = db
  .query(
    "SELECT type, name, sql FROM src.sqlite_master WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%' ORDER BY type = 'table' DESC, name",
  )
  .all() as { type: string; name: string; sql: string }[];
for (const { sql } of schema) {
  db.run(sql);
}

// Resolve the airports and build the area around them
const airportList = airports.map(a => `'${a.replaceAll("'", "''")}'`).join(", ");
const found = db
  .query(
    `SELECT airport_identifier AS ident, airport_ref_latitude AS lat, airport_ref_longitude AS long FROM src.tbl_pa_airports WHERE airport_identifier IN (${airportList})`,
  )
  .all() as { ident: string; lat: number; long: number }[];

const missing = airports.filter(a => !found.some(f => f.ident === a));
if (missing.length > 0) {
  console.error(`[-] Airports not found in the source database: ${missing.join(", ")}`);
  process.exit(1);
}

/// SQL condition matching points within the area around any of the airports
const inArea = (lat: string, long: string) =>
  "(" +
  found
    .map(
      a =>
        `(${lat} BETWEEN ${a.lat - AREA_MARGIN_DEGREES} AND ${a.lat + AREA_MARGIN_DEGREES} AND ${long} BETWEEN ${a.long - AREA_MARGIN_DEGREES} AND ${a.long + AREA_MARGIN_DEGREES})`,
    )
    .join(" OR ") +
  ")";

/// Keeps every row of a group (e.g. all segments of an airway) when any row of the group is within the area
const wholeGroupsInArea = (table: string, key: string[], lat: string, long: string) => {
  const keyExpr = key.join(" || '|' || ");
  return `${keyExpr} IN (SELECT ${keyExpr} FROM src.${table} WHERE ${inArea(lat, long)})`;
};

const airportFilter = `airport_identifier IN (${airportList})`;

/// All airports within the area are included (with their runways) so range queries return realistic results
const airportsInAreaFilter = `airport_identifier IN (SELECT airport_identifier FROM src.tbl_pa_airports WHERE ${inArea("airport_ref_latitude", "airport_ref_longitude")}) OR ${airportFilter}`;

/// Row filter per table. Tables not listed (header, cruising tables, grid MORA) are copied in full.
const filters: Record<string, string> = {
  // Airport data
  tbl_pa_airports: airportsInAreaFilter,
  tbl_pg_runways: airportsInAreaFilter,
  tbl_pb_gates: airportFilter,
  tbl_pd_sids: airportFilter,
  tbl_pe_stars: airportFilter,
  tbl_pf_iaps: airportFilter,
  tbl_pi_localizers_glideslopes: airportFilter,
  tbl_pm_localizer_marker: airportFilter,
  tbl_pn_terminal_ndbnavaids: airportFilter,
  tbl_pp_pathpoint: airportFilter,
  tbl_ps_airport_msa: airportFilter,
  tbl_pt_gls: airportFilter,
  tbl_pv_airport_communication: airportFilter,
  tbl_pc_terminal_waypoints: `region_code IN (${airportList})`,
  // Enroute points
  tbl_d_vhfnavaids: inArea("navaid_latitude", "navaid_longitude"),
  tbl_db_enroute_ndbnavaids: inArea("navaid_latitude", "navaid_longitude"),
  tbl_ea_enroute_waypoints: inArea("waypoint_latitude", "waypoint_longitude"),
  tbl_ep_holdings: inArea("waypoint_latitude", "waypoint_longitude"),
  tbl_ev_enroute_communication: inArea("latitude", "longitude"),
  // Enroute shapes, kept whole
  tbl_er_enroute_airways: wholeGroupsInArea(
    "tbl_er_enroute_airways",
    ["route_identifier"],
    "waypoint_latitude",
    "waypoint_longitude",
  ),
  tbl_eu_enroute_airway_restriction: wholeGroupsInArea(
    "tbl_er_enroute_airways",
    ["route_identifier"],
    "waypoint_latitude",
    "waypoint_longitude",
  ),
  tbl_uc_controlled_airspace: wholeGroupsInArea(
    "tbl_uc_controlled_airspace",
    ["icao_code", "airspace_center", "multiple_code"],
    "latitude",
    "longitude",
  ),
  tbl_ur_restrictive_airspace: wholeGroupsInArea(
    "tbl_ur_restrictive_airspace",
    ["icao_code", "restrictive_airspace_designation", "multiple_code"],
    "latitude",
    "longitude",
  ),
  tbl_uf_fir_uir: wholeGroupsInArea("tbl_uf_fir_uir", ["fir_uir_identifier"], "fir_uir_latitude", "fir_uir_longitude"),
};

console.info(`[*] Generating mock navigation data for ${airports.join(", ")}`);

for (const { name } of schema.filter(s => s.type === "table")) {
  const where = filters[name];
  db.run(`INSERT INTO main.${name} SELECT * FROM src.${name}${where ? ` WHERE ${where}` : ""}`);
  const { count } = db.query(`SELECT count(*) AS count FROM main.${name}`).get() as { count: number };
  console.info(`    ${name.padEnd(36)} ${count}`);
}

db.run("DETACH DATABASE src");
db.run("VACUUM");
db.close();

copyFileSync(join(sourceDir, "cycle.json"), outCycle);

console.info(`[+] Wrote ${outDb} (${(statSync(outDb).size / 1024).toFixed(0)} KB) and ${outCycle}`);
