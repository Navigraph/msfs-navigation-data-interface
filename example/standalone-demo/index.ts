import { loadNavigationData } from "./src/loadNavigationData";

/// Must be one of the airports the mock data was generated for (MMUN, MMMD and MSLP by default)
const AIRPORT = (process.argv[2] ?? "MMUN").toUpperCase();

const { navigationDataInterface, transport } = await loadNavigationData().catch((error: unknown) => {
  console.error("[-]", error instanceof Error ? error.message : error);
  process.exit(1);
});

try {
  const info = await navigationDataInterface.get_database_info();
  console.info(`Database: AIRAC ${info.airac_cycle} (${info.effective_from_to.join(" - ")})`);

  const status = await navigationDataInterface.get_navigation_data_install_status();
  console.info(`Install status: ${status.status}, path: ${status.installedPath}`);

  const airport = await navigationDataInterface.get_airport(AIRPORT);
  console.info(`\n${airport.ident}: ${airport.name} (${airport.location.lat}, ${airport.location.long})`);

  const runways = await navigationDataInterface.get_runways_at_airport(AIRPORT);
  console.info(`Runways: ${runways.map(r => r.ident).join(", ")}`);

  const departures = await navigationDataInterface.get_departures_at_airport(AIRPORT);
  const approaches = await navigationDataInterface.get_approaches_at_airport(AIRPORT);
  console.info(`Departures: ${departures.length}, approaches: ${approaches.length}`);

  const nearby = await navigationDataInterface.get_airports_in_range(airport.location, 50);
  console.info(`Airports within 50 NM: ${nearby.map(a => a.ident).join(", ")}`);

  const rows = await navigationDataInterface.execute_sql<{ airport_identifier: string; airport_name: string }>(
    "SELECT airport_identifier, airport_name FROM tbl_pa_airports ORDER BY airport_identifier LIMIT 5",
    [],
  );
  console.info("\nexecute_sql:");
  rows.forEach(row => console.info(`  ${row.airport_identifier}  ${row.airport_name}`));
} catch (error) {
  console.error("[-]", error instanceof Error ? error.message : error);
  process.exitCode = 1;
} finally {
  // The transport drives the module on an interval, which would otherwise keep the process alive
  transport.dispose();
}
