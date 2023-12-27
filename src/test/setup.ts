import { mkdirSync, readFileSync } from "node:fs"
import { argv, env } from "node:process"
import * as path from "path"
import random from "random-bigint"
import { v4 } from "uuid"
import { WASI } from "wasi"

/**
 * struct sGaugeDrawData
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

enum PanelService {
  PANEL_SERVICE_POST_QUERY = 1,
  PANEL_SERVICE_PRE_INSTALL = 2,
  PANEL_SERVICE_POST_INSTALL = 3,
  PANEL_SERVICE_PRE_INITIALIZE = 4,
  PANEL_SERVICE_POST_INITIALIZE = 5,
  PANEL_SERVICE_PRE_UPDATE = 6,
  PANEL_SERVICE_POST_UPDATE = 7,
  PANEL_SERVICE_PRE_GENERATE = 8,
  PANEL_SERVICE_POST_GENERATE = 9,
  PANEL_SERVICE_PRE_DRAW = 10,
  PANEL_SERVICE_POST_DRAW = 11,
  PANEL_SERVICE_PRE_KILL = 12,
  PANEL_SERVICE_POST_KILL = 13,
  PANEL_SERVICE_CONNECT_TO_WINDOW = 14,
  PANEL_SERVICE_DISCONNECT = 15,
  PANEL_SERVICE_PANEL_OPEN = 16,
  PANEL_SERVICE_PANEL_CLOSE = 17,
}

let instance: WebAssembly.Instance

const wasmRegisteredEvents = new Map<string, [(args_pointer: number, args_size: number, ctx: number) => void, number]>()
const jsRegisteredEvents = new Map<string, (jsonArgs: string) => void>()

let memoryBuffer: Uint8Array

function readString(pointer: number): string {
  let lastChar = pointer

  while (memoryBuffer[lastChar] !== 0) {
    lastChar++
  }

  return new TextDecoder().decode(memoryBuffer.slice(pointer, lastChar))
}

function malloc(size: number): number {
  const pointer = instance.exports.malloc(size) as number
  memoryBuffer = new Uint8Array(instance.exports.memory.buffer)
  return pointer
}

function writeString(value: string): [number, number] {
  const encoded = new TextEncoder().encode(value)

  const pointer = malloc(encoded.length, memoryBuffer)

  memoryBuffer.set(encoded, pointer)

  return [pointer, encoded.length]
}

class CommBusListener {
  callWasm(name: string, jsonBuf: string) {
    const data = wasmRegisteredEvents.get(name)

    if (!data) return

    const [func, t] = data

    const [args, size] = writeString(jsonBuf)

    func(args, size, t)
  }

  on(eventName: string, callback: (args: string) => void) {
    jsRegisteredEvents.set(eventName, callback)
  }
}

global.RegisterCommBusListener = function RegisterCommBusListener(callback?: () => void) {
  if (callback) setTimeout(callback, 1)

  return new CommBusListener()
}

global.Utils = {
  generateGUID() {
    return v4()
  },
}

const wasi = new WASI({
  version: "preview1",
  args: argv,
  env,
  preopens: {
    "\\work": "./test_out",
  },
})

const wasm = new WebAssembly.Module(readFileSync("./out/msfs_navdata_interface.wasm"))

let table: WebAssembly.Table

const promiseResults = new Map<bigint, [number, number]>()

instance = new WebAssembly.Instance(wasm, {
  wasi_snapshot_preview1: wasi.wasiImport,
  env: {
    fsCommBusCall: (eventName: number, args: number) => {
      jsRegisteredEvents.get(readString(eventName))?.(readString(args))
      return true
    },
    fsCommBusUnregister: (eventNamePointer: number) => {
      const eventName = readString(eventNamePointer)
      wasmRegisteredEvents.delete(eventName)
      return 0
    },
    fsCommBusRegister: (eventNamePointer: number, callback: number, t: number) => {
      const eventName = readString(eventNamePointer)
      const func = table.get(callback) as () => void

      wasmRegisteredEvents.set(eventName, [func, t])

      return true
    },
    fsNetworkHttpRequestGetDataSize: (requestId: bigint) => {
      const data = promiseResults.get(requestId)
      if (!data) return 0

      return data[1]
    },
    fsNetworkHttpRequestGetData: (requestId: bigint) => {
      const data = promiseResults.get(requestId)
      if (!data) return 0

      return data[0]
    },
    fsNetworkHttpRequestGet: (urlPointer: number, paramPointer: number, callback: number, t: number) => {
      const url = readString(urlPointer)

      const requestId: bigint = random(32) // Setting it to 64 does... strange things

      fetch(url)
        .then(result => result.blob())
        .then(async blob => {
          const data = new Uint8Array(await blob.arrayBuffer())

          const pointer = malloc(data.length)

          memoryBuffer.set(data, pointer)
          promiseResults.set(requestId, [pointer, data.length])

          const func = table.get(callback) as () => void
          func(requestId, 200, t)
        })
        .catch(err => {
          console.error(err)
        })

      return requestId
    },
  },
})

memoryBuffer = new Uint8Array(instance.exports.memory.buffer)
table = instance.exports.__indirect_function_table

wasi.initialize(instance)

instance.exports.navdata_interface_gauge_callback("", PanelService.PANEL_SERVICE_PRE_INSTALL, () => {})
instance.exports.navdata_interface_gauge_callback("", PanelService.PANEL_SERVICE_POST_INITIALIZE, () => {})

const drawRate = 30

let runLifecycle = true

async function lifeCycle() {
  while (runLifecycle) {
    await new Promise(resolve => setTimeout(resolve, 1000 / drawRate))

    const floats = new Uint8Array(new Float64Array([0, 0, 0, 1 / drawRate]).buffer) // First 4 64 bit doubles of sGaugeDrawData
    const ints = new Uint8Array(new Int32Array([0, 0, 0, 0]).buffer) // Last 4 32 bit ints of sGaugeDrawData

    const array = new Uint8Array([...floats, ...ints])

    const pointer = malloc(array.length)

    memoryBuffer.set(array, pointer)

    instance.exports.navdata_interface_gauge_callback("", PanelService.PANEL_SERVICE_PRE_DRAW, pointer)

    instance.exports.free(pointer)
  }
}

void lifeCycle()

afterAll(() => {
  runLifecycle = false
})
