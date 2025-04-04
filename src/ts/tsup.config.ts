import { defineConfig } from "tsup";

export default defineConfig({
  clean: true,
  entry: { "msfs-navigation-data-interface": "index.ts" },
  format: "esm",
  target: "es6",
  dts: true,
});
