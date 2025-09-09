import { $ } from "bun";
import { constants, copyFile, existsSync, mkdirSync, readFileSync, rmdirSync } from "node:fs";
import { dirname, join, normalize, resolve } from "node:path";

/// The type returned from the `cargo-msfs info -f` command
interface InstalledSdkVersions {
  versions: { sim: "Msfs2020"; up_to_date: boolean; installed?: string; latest: string }[];
}

/// The docker image name
const IMAGE_NAME = "navigation-data-interface-wasm-build";

/// Find workspace root
function findWorkspaceRoot() {
  let previous = null;
  let current = normalize(process.cwd());

  do {
    // Try reading a package.json in this directory
    const packageJson = join(current, "package.json");

    if (existsSync(packageJson)) {
      const manifest = JSON.parse(readFileSync(packageJson, "utf-8")) as { workspaces?: string[] };

      // Check if there is workspaces, meaning this is root
      if (manifest.workspaces) {
        return current;
      }
    }

    // Iterate up
    previous = current;
    current = dirname(current);
  } while (current !== previous);

  return null;
}

// Get workspace root for docker commands
const workspaceRoot = findWorkspaceRoot();
if (!workspaceRoot) {
  console.error("[-] Unable to find workspace root. Exiting...");
  process.exit(1);
}

// Ensure docker is installed and available
await $`docker ps`.quiet().catch(() => {
  console.error("[-] Docker is not installed or not running");
  process.exit(1);
});

// Ensure image is built
await $`docker image inspect ${IMAGE_NAME}:latest`.quiet().catch(async () => {
  const dockerfilePath = resolve(workspaceRoot, "Dockerfile");
  console.info(`[*] Building '${IMAGE_NAME}' image from ${dockerfilePath}`);
  await $`docker build --no-cache -t ${IMAGE_NAME} -f ${dockerfilePath} .`;
});

// Ensure SDKs are up to date, rebuilding if needed
const installedSdks = JSON.parse(
  await $`docker run --rm ${IMAGE_NAME} bash -c "cargo-msfs info -f"`.text(),
) as InstalledSdkVersions;
if (installedSdks.versions.some(v => !v.up_to_date)) {
  console.info("[*] Updating SDK in Docker image...");
  await $`docker build --build-arg CACHEBUST=${Date.now()} -t ${IMAGE_NAME} -f ${resolve(workspaceRoot, "Dockerfile")} .`;
}

// Clear out dir
const outDir = resolve(workspaceRoot, "dist/wasm");
const panelDir = resolve(
  workspaceRoot,
  "example/aircraft/PackageSources/SimObjects/Airplanes/Navigraph_Navigation_Data_Interface_Aircraft/panel",
);

if (existsSync(outDir)) rmdirSync(outDir, { recursive: true });

// The work directory, relative to workspace root
const relativeWorkdDir = process.cwd().replace(workspaceRoot, "").replaceAll("\\", "/");

// Build the 2020 version
const simVersion = "2020";

console.info(`[*] Building for ${simVersion}`);

// Create the subfolder
const simDir = join(outDir, simVersion);
const relativeSimDir = simDir.replace(workspaceRoot, "").replaceAll("\\", "/");
mkdirSync(simDir, { recursive: true });

// Run cargo-msfs
await $`docker run \
  --rm -t \
  --name msfs-${simVersion}-wasm-builder \
  -v ${workspaceRoot}:/workspace \
  -w /workspace${relativeWorkdDir} \
  -e CARGO_TARGET_DIR=/workspace/targets/${simVersion} \
  ${IMAGE_NAME} \
    bash -c "cargo-msfs build msfs${simVersion} -i ./src/wasm -o ./${relativeSimDir}/msfs_navigation_data_interface.wasm \
1> >(sed \"s/^/[\x1b[34m${simVersion}\\x1b[0m]/\") \
2> >(sed \"s/^/[\x1b[34m${simVersion}\\x1b[0m]/\" >&2)"`.catch((err: { exitCode?: number; stderr?: Buffer }) => {
  console.error(`[-] Error building for ${simVersion}: ${err.exitCode} ${err.stderr?.toString()}`);
  process.exit(1);
});

copyFile(
  `${join(simDir, "msfs_navigation_data_interface.wasm")}`,
  `${join(panelDir, "msfs_navigation_data_interface.wasm")}`,
  constants.COPYFILE_FICLONE,
  err => {
    if (err) {
      console.error("[-] Wasm module copy failed ");
      process.exit(1);
    }

    console.info(`[*] Copying WASM module to aircraft panel folder`);
  },
);
