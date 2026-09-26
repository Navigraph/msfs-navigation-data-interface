import { NavigraphFunction, RawNavigraphEvent } from "../interface/NavigationDataInterfaceTypes";
import { NavigationDataTransport } from "./NavigationDataTransport";
import { WasmMessageRouter } from "./WasmMessageRouter";

/**
 * The standalone build of the WASM module (`bun run build:wasm:standalone`), as a URL to fetch, the module bytes, a compiled module,
 * or a (pending) fetch response
 */
export type StandaloneWasmSource = string | URL | BufferSource | WebAssembly.Module | Response | PromiseLike<Response>;

export interface StandaloneTransportOptions {
  /** The standalone build of the WASM module, which serves mock data */
  wasm: StandaloneWasmSource;
  /** Milliseconds between updates of the module, the equivalent of a sim frame (default: 16) */
  updateIntervalMs?: number;
}

/** The exports of the standalone WASM module (see `src/wasm/src/standalone.rs`) */
interface StandaloneExports {
  memory: WebAssembly.Memory;
  _initialize?: () => void;
  navigraph_alloc(len: number): number;
  navigraph_call_function(ptr: number, len: number): void;
  navigraph_update(): void;
}

/** WASI errno values */
const ERRNO_SUCCESS = 0;
const ERRNO_BADF = 8;
const ERRNO_NOSYS = 52;

/** WASI clock IDs */
const CLOCK_REALTIME = 0;

/**
 * A transport which runs the standalone build of the WASM module outside the sim, serving mock data.
 *
 * @remarks
 * Calls are passed to the module with the same payloads as the CommBus, so they are dispatched to the same functions in the module,
 * which serve mock data instead of querying the navigation database.
 */
export class StandaloneTransport implements NavigationDataTransport {
  private readonly router = new WasmMessageRouter();
  private readonly ready: Promise<StandaloneExports>;
  private exports: StandaloneExports | null = null;
  private updateTimer: ReturnType<typeof setInterval> | null = null;
  private nextId = 0;

  /** Messages sent by the module during the current export call, delivered once it returns */
  private pendingMessages: [channel: string, data: string][] = [];

  constructor(options: StandaloneTransportOptions) {
    this.ready = this.load(options.wasm).then(exports => {
      this.exports = exports;
      this.updateTimer = setInterval(() => this.runExport(e => e.navigraph_update()), options.updateIntervalMs ?? 16);
      return exports;
    });

    this.ready.catch((error: unknown) => {
      const err = error instanceof Error ? error : new Error(String(error));
      console.error("[NAVIGRAPH]: Unable to load standalone module", err);
      this.router.rejectAll(err);
    });
  }

  public async call<T>(functionName: keyof typeof NavigraphFunction, args: unknown): Promise<T> {
    await this.ready;

    const { payload, result } = this.router.createCall<T>(`standalone-${this.nextId++}`, functionName, args);

    this.runExport(exports => {
      const bytes = new TextEncoder().encode(payload);
      const ptr = exports.navigraph_alloc(bytes.length);
      new Uint8Array(exports.memory.buffer, ptr, bytes.length).set(bytes);
      // The module takes ownership of the buffer
      exports.navigraph_call_function(ptr, bytes.length);
    });

    return result;
  }

  public onEvent(handler: (event: RawNavigraphEvent) => void): void {
    this.router.setEventHandler(handler);
  }

  /**
   * Stops updating the module. Pending calls will not resolve.
   */
  public dispose(): void {
    if (this.updateTimer !== null) {
      clearInterval(this.updateTimer);
      this.updateTimer = null;
    }
  }

  /**
   * Calls into the module, then delivers the messages it sent.
   *
   * @remarks
   * Delivery is deferred so that handlers calling back into the transport (e.g. from an `onReady` callback) don't re-enter the module.
   */
  private runExport(fn: (exports: StandaloneExports) => void): void {
    if (!this.exports) {
      return;
    }

    fn(this.exports);

    const messages = this.pendingMessages;
    this.pendingMessages = [];
    messages.forEach(([channel, data]) => this.router.handleMessage(channel, data));
  }

  private async load(source: StandaloneWasmSource): Promise<StandaloneExports> {
    const module = await compileModule(source);

    let memory: WebAssembly.Memory | null = null;
    const getMemory = () => {
      if (!memory) throw new Error("Standalone module memory accessed before instantiation");
      return memory;
    };
    const decoder = new TextDecoder();
    const readString = (ptr: number, len: number) => decoder.decode(new Uint8Array(getMemory().buffer, ptr, len));

    const navigraph = {
      send_message: (channelPtr: number, channelLen: number, dataPtr: number, dataLen: number) => {
        this.pendingMessages.push([readString(channelPtr, channelLen), readString(dataPtr, dataLen)]);
      },
    };

    const imports = buildImports(module, { navigraph, wasi_snapshot_preview1: createWasiShim(getMemory) });
    const instance = await WebAssembly.instantiate(module, imports);
    const exports = instance.exports as unknown as StandaloneExports;

    for (const name of ["memory", "navigraph_alloc", "navigraph_call_function", "navigraph_update"] as const) {
      if (!(name in exports)) {
        throw new Error(`The WASM module does not export '${name}'. Is it the standalone build?`);
      }
    }

    memory = exports.memory;
    exports._initialize?.();

    return exports;
  }
}

async function compileModule(source: StandaloneWasmSource): Promise<WebAssembly.Module> {
  if (source instanceof WebAssembly.Module) {
    return source;
  }

  if (typeof source === "string" || source instanceof URL || source instanceof Response || isPromiseLike(source)) {
    const response = await (typeof source === "string" || source instanceof URL ? fetch(source) : source);
    if (!response.ok) {
      throw new Error(`Unable to fetch standalone module from ${response.url}: ${response.status}`);
    }
    return WebAssembly.compile(await response.arrayBuffer());
  }

  return WebAssembly.compile(source);
}

function isPromiseLike(value: unknown): value is PromiseLike<Response> {
  return typeof (value as PromiseLike<Response> | undefined)?.then === "function";
}

type ImportFunctions = Record<string, (...args: never[]) => unknown>;

/**
 * Builds the import object for the module. Any WASI function which the shim doesn't implement returns `ENOSYS`.
 */
function buildImports(
  module: WebAssembly.Module,
  provided: { navigraph: ImportFunctions; wasi_snapshot_preview1: ImportFunctions },
): WebAssembly.Imports {
  const imports: Record<string, ImportFunctions> = { navigraph: {}, wasi_snapshot_preview1: {} };

  for (const { module: moduleName, name, kind } of WebAssembly.Module.imports(module)) {
    if (kind !== "function" || !(moduleName === "navigraph" || moduleName === "wasi_snapshot_preview1")) {
      throw new Error(`The standalone module has an unsupported import '${moduleName}.${name}'`);
    }

    const fn = provided[moduleName][name];
    if (fn) {
      imports[moduleName]![name] = fn;
    } else if (moduleName === "wasi_snapshot_preview1") {
      imports[moduleName]![name] = () => ERRNO_NOSYS;
    } else {
      throw new Error(`The standalone module imports '${moduleName}.${name}', which is not provided`);
    }
  }

  return imports as WebAssembly.Imports;
}

/**
 * A minimal WASI implementation, covering what the Rust standard library needs: stdout/stderr, clocks and randomness
 */
function createWasiShim(getMemory: () => WebAssembly.Memory): ImportFunctions {
  const view = () => new DataView(getMemory().buffer);
  const decoder = new TextDecoder();
  const lineBuffers: Record<number, string> = { 1: "", 2: "" };

  return {
    fd_write: (fd: number, iovs: number, iovsLen: number, nwritten: number) => {
      if (fd !== 1 && fd !== 2) {
        return ERRNO_BADF;
      }

      const dv = view();
      let written = 0;
      let text = "";
      for (let i = 0; i < iovsLen; i++) {
        const ptr = dv.getUint32(iovs + i * 8, true);
        const len = dv.getUint32(iovs + i * 8 + 4, true);
        text += decoder.decode(new Uint8Array(getMemory().buffer, ptr, len));
        written += len;
      }

      // Print complete lines, keeping any partial line until the rest arrives
      const lines = (lineBuffers[fd] + text).split("\n");
      lineBuffers[fd] = lines.pop() ?? "";
      lines.forEach(line => (fd === 1 ? console.info(line) : console.error(line)));

      dv.setUint32(nwritten, written, true);
      return ERRNO_SUCCESS;
    },
    clock_time_get: (clockId: number, _precision: unknown, time: number) => {
      // Written as two 32 bit halves, as the package targets ES6 (no BigInt). Sub-microsecond precision is lost, which is fine here.
      const nanos = (clockId === CLOCK_REALTIME ? Date.now() : performance.now()) * 1e6;
      const high = Math.floor(nanos / 2 ** 32);
      const dv = view();
      dv.setUint32(time, Math.floor(nanos - high * 2 ** 32), true);
      dv.setUint32(time + 4, high, true);
      return ERRNO_SUCCESS;
    },
    random_get: (buf: number, len: number) => {
      // getRandomValues is limited to 65536 bytes per call
      for (let offset = 0; offset < len; offset += 65536) {
        crypto.getRandomValues(new Uint8Array(getMemory().buffer, buf + offset, Math.min(65536, len - offset)));
      }
      return ERRNO_SUCCESS;
    },
    environ_sizes_get: (count: number, size: number) => {
      view().setUint32(count, 0, true);
      view().setUint32(size, 0, true);
      return ERRNO_SUCCESS;
    },
    environ_get: () => ERRNO_SUCCESS,
    args_sizes_get: (count: number, size: number) => {
      view().setUint32(count, 0, true);
      view().setUint32(size, 0, true);
      return ERRNO_SUCCESS;
    },
    args_get: () => ERRNO_SUCCESS,
    sched_yield: () => ERRNO_SUCCESS,
    proc_exit: (code: number) => {
      throw new Error(`Standalone module exited with code ${code}`);
    },
  };
}
