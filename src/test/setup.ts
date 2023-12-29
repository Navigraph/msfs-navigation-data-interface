import { readFileSync } from "node:fs"
import { argv, env } from "node:process"
import random from "random-bigint"
import { v4 } from "uuid"
import { WASI } from "wasi"
import { NavigraphNavdataInterface } from "../js"
import { WEBASSEMBLY_PATH, WORK_FOLDER_PATH } from "./constants"
import "dotenv/config"

enum PanelService {
  POST_QUERY = 1,
  PRE_INSTALL = 2,
  POST_INSTALL = 3,
  PRE_INITIALIZE = 4,
  POST_INITIALIZE = 5,
  PRE_UPDATE = 6,
  POST_UPDATE = 7,
  PRE_GENERATE = 8,
  POST_GENERATE = 9,
  PRE_DRAW = 10,
  POST_DRAW = 11,
  PRE_KILL = 12,
  POST_KILL = 13,
  CONNECT_TO_WINDOW = 14,
  DISCONNECT = 15,
  PANEL_OPEN = 16,
  PANEL_CLOSE = 17,
}

type WasmInstance = {
  exports: {
    navdata_interface_gauge_callback: (fsContext: bigint, serviceId: PanelService, dataPointer: number) => void
    malloc: (size: number) => number
    free: (pointer: number) => void
    memory: WebAssembly.Memory
    __indirect_function_table: WebAssembly.Table
  }
}

// eslint-disable-next-line prefer-const
let wasmInstance: WasmInstance // The instance of the wasm module

type WasmEventCallback = (argsPointer: number, argsSize: number, ctx: number) => void

/**
 * The events registered by wasm CommBus
 * [eventName: string, callback, ctx]
 * The third value, ctx value must be passed to the callback when called
 */
let wasmRegisteredEvents: [string, WasmEventCallback, number][] = []

type JSEventCallback = (args: string) => void

/**
 * The events registered by js CommBus
 * [eventName, callback]
 */
const jsRegisteredEvents: [string, JSEventCallback][] = []

/**
 * A Uint8Array created from the wasm instance memory buffer
 * This is how one should access the wasm memory
 */
let memoryBuffer: Uint8Array

/**
 * Allocate memory in the wasm instance. This memory can be accessed by slicing the `memoryBuffer` with the pointer and the size.
 * @param size - The number of bytes to allocate
 * @returns A pointer to the allocated memory.
 */
function malloc(size: number): number {
  const pointer = wasmInstance.exports.malloc(size)
  memoryBuffer = new Uint8Array(wasmInstance.exports.memory.buffer as ArrayBufferLike)
  return pointer
}

/**
 * Reads a CString from the `memoryBuffer`.
 *
 * The string terminates when a null byte is found
 * @param pointer - The pointer to the location in `memoryBuffer` where the string is stored
 * @returns The string from memory using `TextDecoder`
 */
function readString(pointer: number): string {
  let lastChar = pointer

  while (memoryBuffer[lastChar] !== 0) {
    lastChar++
  }

  return new TextDecoder().decode(memoryBuffer.slice(pointer, lastChar))
}

/**
 * Writes a string to the `memoryBuffer` which can be read by wasm.
 * @param value - The string to write to memory
 * @returns A tuple containing the pointer to the string and the size of the string
 */
function writeString(value: string): [number, number] {
  const encoded = new TextEncoder().encode(value)

  const pointer = malloc(encoded.length)

  memoryBuffer.set(encoded, pointer)

  return [pointer, encoded.length]
}

class CommBusListener {
  callWasm(name: string, args: string) {
    const events = wasmRegisteredEvents.filter(([eventName]) => eventName === name)

    events.forEach(([, func, ctx]) => {
      const [pointer, size] = writeString(args)

      func(pointer, size, ctx)
    })
  }

  on(eventName: string, callback: JSEventCallback) {
    if (!jsRegisteredEvents.find(([name, func]) => name === eventName && func === callback)) {
      jsRegisteredEvents.push([eventName, callback])
    }
  }
}

// @ts-ignore The CommBusListener we return only needs to implement the CommBus functions we use
global.RegisterCommBusListener = function RegisterCommBusListener(callback?: () => void) {
  if (callback) setTimeout(callback, 1)

  return new CommBusListener()
}

// @ts-ignore Currently we only use generateGUID
global.Utils = {
  generateGUID() {
    return v4()
  },
}

const wasiSystem = new WASI({
  version: "preview1",
  args: argv,
  env,
  preopens: {
    "\\work": WORK_FOLDER_PATH,
  },
})

// Read the wasm from the file, and compile it into a module
const wasmModule = new WebAssembly.Module(readFileSync(WEBASSEMBLY_PATH))

// eslint-disable-next-line prefer-const
let wasmFunctionTable: WebAssembly.Table // The table of callback functions in the wasm module

/**
 * Maps request ids to a tuple of the returned data's pointer, and the data's size
 */
const promiseResults = new Map<bigint, [number, number]>()

wasmInstance = new WebAssembly.Instance(wasmModule, {
  wasi_snapshot_preview1: wasiSystem.wasiImport,
  env: {
    fsCommBusCall: (eventNamePointer: number, args: number) => {
      const eventName = readString(eventNamePointer)

      const events = jsRegisteredEvents.filter(([name]) => name === eventName)

      events.forEach(data => {
        const func = data[1] // For some reason destructuing the array in args causes a type error...
        func(readString(args))
      })

      return true
    },
    fsCommBusUnregister: (eventNamePointer: number, callback: number) => {
      const eventName = readString(eventNamePointer)
      const func = wasmFunctionTable.get(callback) as WasmEventCallback

      wasmRegisteredEvents = wasmRegisteredEvents.filter(([name, func1]) => name !== eventName || func1 !== func)
      return 0
    },
    fsCommBusRegister: (eventNamePointer: number, callback: number, ctx: number) => {
      const eventName = readString(eventNamePointer)
      const func = wasmFunctionTable.get(callback) as WasmEventCallback

      if (!wasmRegisteredEvents.find(([name, func1]) => name === eventName && func1 === func)) {
        wasmRegisteredEvents.push([eventName, func, ctx])
      }

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
    fsNetworkHttpRequestGet: (urlPointer: number, paramPointer: number, callback: number, ctx: number) => {
      const url = readString(urlPointer)

      const requestId: bigint = random(32) // Setting it to 64 does... strange things

      // Currently the only network request is for the navdata zip which is downloaded as a blob
      fetch(url)
        .then(result => result.blob())
        .then(async blob => {
          const data = new Uint8Array(await blob.arrayBuffer())

          const pointer = malloc(data.length)

          memoryBuffer.set(data, pointer)
          promiseResults.set(requestId, [pointer, data.length])

          const func = wasmFunctionTable.get(callback) as (requestId: bigint, statusCode: number, ctx: number) => void
          func(requestId, 200, ctx)
        })
        .catch(err => {
          console.error(err)
        })

      return requestId
    },
  },
}) as WasmInstance

// Initially assign `memoryBuffer` to a new Uint8Array linked to the exported memoryBuffer
memoryBuffer = new Uint8Array(wasmInstance.exports.memory.buffer)
wasmFunctionTable = wasmInstance.exports.__indirect_function_table

wasiSystem.initialize(wasmInstance)

const fsContext = BigInt(0)

// Run the initialisation functions to setup the gauge
wasmInstance.exports.navdata_interface_gauge_callback(fsContext, PanelService.PRE_INSTALL, 0)
wasmInstance.exports.navdata_interface_gauge_callback(fsContext, PanelService.POST_INITIALIZE, 0)

const drawRate = 30

let runLifecycle = true

/**
 * Runs the life cycle loop for the gauge
 * This only calls the PANEL_SERVICE_PRE_DRAW as of now as its the only function our wasm instance uses
 * This will run until `runLifeCycle` is set to false
 */
async function lifeCycle() {
  while (runLifecycle) {
    await new Promise(resolve => setTimeout(resolve, 1000 / drawRate))

    const floats = new Uint8Array(new Float64Array([0, 0, 0, 1 / drawRate]).buffer) // First 4 64 bit doubles of sGaugeDrawData
    const ints = new Uint8Array(new Int32Array([0, 0, 0, 0]).buffer) // Last 4 32 bit ints of sGaugeDrawData

    const array = new Uint8Array([...floats, ...ints])

    const pointer = malloc(array.length)

    memoryBuffer.set(array, pointer)

    wasmInstance.exports.navdata_interface_gauge_callback(fsContext, PanelService.PRE_DRAW, pointer)

    wasmInstance.exports.free(pointer)
  }
}

// This will run once for each test file
beforeAll(async () => {
  const navdataInterface = new NavigraphNavdataInterface()

  const downloadUrl = process.env.NAVDATA_SIGNED_URL

  if (!downloadUrl) {
    throw new Error("Please specify the env var `NAVDATA_SIGNED_URL`")
  }

  // Download navdata to a unique folder to prevent clashes
  const path = v4()

  await navdataInterface.downloadNavdata(downloadUrl, path)
  await navdataInterface.setActiveDatabase(path)
}, 10000)

void lifeCycle()

// Cancel the lifeCycle after all tests have completed
afterAll(() => {
  runLifecycle = false
})
