import { DataStore } from "@microsoft/msfs-sdk"
import { initializeApp, NavigraphApp, Scope } from "navigraph/app"
import { getAuth } from "navigraph/auth"
import { getChartsAPI } from "navigraph/charts"
import { getPackagesAPI } from "navigraph/packages"

const config: NavigraphApp = {
  clientId: process.env.NG_CLIENT_ID,
  clientSecret: process.env.NG_CLIENT_SECRET,
  scopes: [Scope.FMSDATA, Scope.CHARTS],
}

if (!config.clientId || config.clientId.includes("<")) {
  console.error("Please add your client credentials in an .env file in the root of the project.")
}

initializeApp(config)

// Wait for DataStorage ready event before initializing SDK
const dataStoreInit = new Promise<void>(res => {
  const lis = RegisterViewListener("JS_LISTENER_DATASTORAGE", () => {
    res()
    lis.unregister()
  })
})

const isNavigraphClient = config.clientId.includes("navigraph")
const clientPrefix = isNavigraphClient ? "NG" : config.clientId.toUpperCase().replace("-", "_") + "_NG"

export const AUTH_STORAGE_KEYS = {
  accessToken: `${clientPrefix}_ACCESS_TOKEN`,
  refreshToken: `${clientPrefix}_REFRESH_TOKEN`,
} as const

export const auth = getAuth({
  storage: {
    getItem: key => dataStoreInit.then(() => DataStore.get(key)?.toString() ?? null),
    setItem: (key, value) => DataStore.set(key, value),
  },
  keys: AUTH_STORAGE_KEYS,
})

export const charts = getChartsAPI()

export const packages = getPackagesAPI()
