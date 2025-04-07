import { $ } from "bun";
import { existsSync, mkdirSync, readFileSync, rmdirSync } from "node:fs";
import { resolve, normalize, join, dirname } from "node:path";

/// The type returned from the `cargo-msfs info -f` command
interface InstalledSdkVersions {
  versions: { sim: "Msfs2020" | "Msfs2024"; up_to_date: boolean; installed?: string; latest: string }[];
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
      const manifest = JSON.parse(readFileSync(packageJson, "utf8"));
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

async function main() {
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
    await $`docker build -t ${IMAGE_NAME} -f ${dockerfilePath} .`;
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
  if (existsSync(outDir)) rmdirSync(outDir, { recursive: true });

  // The work directory, relative to workspace root
  const relativeWorkdDir = process.cwd().replace(workspaceRoot, "").replaceAll("\\", "/");

  // Determine which version(s) to build based on command line argument --version
  const allowedVersions = ["msfs2020", "msfs2024"];
  let versionsToBuild = allowedVersions;

  const versionArgIndex = process.argv.indexOf("--version");
  if (versionArgIndex !== -1 && process.argv[versionArgIndex + 1]) {
    const versionArg = process.argv[versionArgIndex + 1];
    if (versionArg && allowedVersions.includes(versionArg)) {
      versionsToBuild = [versionArg];
    } else {
      console.error(`Invalid version argument: ${versionArg}. Allowed values are ${allowedVersions.join(", ")}`);
      process.exit(1);
    }
  }

  // Build the selected versions
  for (const version of versionsToBuild) {
    console.info(`[*] Building for ${version}`);

    // Create the subfolder
    const simDir = join(outDir, version);
    const relativeSimDir = simDir.replace(workspaceRoot, "").replaceAll("\\", "/");
    mkdirSync(simDir, { recursive: true });

    // Run cargo-msfs
    await $`docker run \
    -v ${workspaceRoot}:/workspace \
    -w /workspace${relativeWorkdDir} \
    ${IMAGE_NAME} \
    bash -c "cargo-msfs build ${version} -i ./src/wasm -o ./${relativeSimDir}/msfs_navigation_data_interface.wasm"`.catch(
      err => process.exit(err.exitCode ?? 1),
    );
  }
}

await main();
