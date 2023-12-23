"use strict"

import { readFile } from "node:fs/promises"
import { argv, env } from "node:process"
import { WASI } from "wasi"

const wasi = new WASI({
  version: "preview1",
  args: argv,
  env,
  preopens: {
    "/work": "C:/",
  },
})

;(async () => {
  const wasm = await WebAssembly.compile(await readFile("../wasm_navdata_interface/out/navdata_interface.wasm"))
  let memoryBuffer: Uint8Array
  let table: WebAssembly.Table

  function readString(pointer: number) {
    let lastChar = pointer

    while (memoryBuffer[lastChar] !== 0) {
      lastChar++
    }

    return new TextDecoder().decode(memoryBuffer.slice(pointer, lastChar))
  }

  const instance = await WebAssembly.instantiate(wasm, {
    wasi_snapshot_preview1: wasi.wasiImport,
    env: {
      fsCommBusCall: (eventName: number, args: number) => {
        console.log(`Called: ${readString(eventName)}, ${readString(args)}`)

        return true
      },
      fsCommBusUnregister: (eventName: number, callback: number) => {
        console.log(`Un Registered: ${eventName} ${callback}`)
        return 0
      },
      fsCommBusRegister: (eventNamePointer: number, callback: number) => {
        const eventName = readString(eventNamePointer)
        const func = table.get(callback) as () => void

        console.log(`Registered: ${eventName} ${func.name}`)
        return true
      },
      fsNetworkHttpRequestGetDataSize: () => {},
      fsNetworkHttpRequestGetData: () => {},
      fsNetworkHttpRequestGet: () => {},
    },
  })

  memoryBuffer = new Uint8Array(instance.exports.memory.buffer)
  table = instance.exports.__indirect_function_table

  wasi.initialize(instance)

  instance.exports.navdata_interface_gauge_callback("", 2, () => {}) // Run PANEL_SERVICE_PRE_INSTALL
  instance.exports.navdata_interface_gauge_callback("", 5, () => {}) // Run PANEL_SERVICE_POST_INITIALIZE

  const drawRate = 30

  // TODO: Extract to seperate lifecycle
  while (true) {
    await new Promise(resolve => setTimeout(resolve, 1000 / drawRate))

    const floats = new Uint8Array(new Float64Array([0, 0, 0, 1 / drawRate]).buffer) // First 4 64 bit doubles of sGaugeDrawData
    const ints = new Uint8Array(new Int32Array([0, 0, 0, 0]).buffer) // Last 4 32 bit ints of sGaugeDrawData

    const array = new Uint8Array([...floats, ...ints])

    const pointer = instance.exports.malloc(array.length) as number

    memoryBuffer.set(array, pointer)

    instance.exports.navdata_interface_gauge_callback("", 10, pointer) // Run PANEL_SERVICE_PRE_DRAW

    instance.exports.free(pointer)
  }
})().catch(() => {})

// TODO: Make this an enum
/**
 * #define PANEL_SERVICE_PRE_QUERY                         0
#define PANEL_SERVICE_POST_QUERY                        1
#define PANEL_SERVICE_PRE_INSTALL                       2       // extra_data = resource_handle
#define PANEL_SERVICE_POST_INSTALL                      3       // extra_data = resource_handle
#define PANEL_SERVICE_PRE_INITIALIZE                    4
#define PANEL_SERVICE_POST_INITIALIZE                   5
#define PANEL_SERVICE_PRE_UPDATE                        6
#define PANEL_SERVICE_POST_UPDATE                       7
#define PANEL_SERVICE_PRE_GENERATE                      8       // extra_data = phase
#define PANEL_SERVICE_POST_GENERATE                     9       // extra_data = phase
#define PANEL_SERVICE_PRE_DRAW                          10
#define PANEL_SERVICE_POST_DRAW                         11
#define PANEL_SERVICE_PRE_KILL                          12
#define PANEL_SERVICE_POST_KILL                         13
#define PANEL_SERVICE_CONNECT_TO_WINDOW                 14      // extra_data = PANEL_WND
#define PANEL_SERVICE_DISCONNECT                        15      // extra_data = PANEL_WND
#define PANEL_SERVICE_PANEL_OPEN                        16
#define PANEL_SERVICE_PANEL_CLOSE                       17

struct sGaugeDrawData
{
	double mx;
	double my;
	double t;
	double dt; // In Seconds
	int winWidth;
	int winHeight;
	int fbWidth;
	int fbHeight;
};
 */
