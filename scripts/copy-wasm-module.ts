import { copyFile } from "node:fs";
import { join, resolve } from "node:path";

const rootDir = resolve(__dirname, "..");

const wasmModuleDir = join(rootDir, "dist/wasm/2020/msfs_navigation_data_interface.wasm");
const panelDir = join(
  rootDir,
  "example/aircraft/PackageSources/SimObjects/Airplanes/Navigraph_Navigation_Data_Interface_Aircraft/panel/msfs_navigation_data_interface.wasm",
);

copyFile(wasmModuleDir, panelDir, err => {
  if (err) {
    console.error(`[-] Wasm module copy failed: ${err.message}`);
    process.exit(1);
  }

  console.info(`[*] Copying WASM module to aircraft panel folder`);
});
