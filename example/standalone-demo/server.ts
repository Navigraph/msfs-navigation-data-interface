import { createApp } from "./src/app.module";
import { loadNavigationData } from "./src/loadNavigationData";

const PORT = Number(process.env.PORT ?? 3000);

const { navigationDataInterface, transport } = await loadNavigationData(process.env.NAVIGRAPH_WASM_PATH).catch(
  (error: unknown) => {
    console.error("[-]", error instanceof Error ? error.message : error);
    process.exit(1);
  },
);

const app = await createApp(navigationDataInterface);
await app.listen(PORT);
console.info(`[+] Listening on http://localhost:${PORT} (API docs at http://localhost:${PORT}/docs)`);

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.once(signal, () => {
    transport.dispose();
    void app.close();
  });
}
