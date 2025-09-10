import { $ } from "bun";

await $`cp dist/wasm/2020/msfs_navigation_data_interface.wasm example/aircraft/PackageSources/SimObjects/Airplanes/Navigraph_Navigation_Data_Interface_Aircraft/panel/msfs_navigation_data_interface.wasm`;
