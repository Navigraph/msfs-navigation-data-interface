import { DataStore } from "@microsoft/msfs-sdk"
import { initializeApp, NavigraphApp, Scope } from "@navigraph/app"
import { getAuth } from "@navigraph/auth"
import { getChartsAPI } from "@navigraph/charts"
import { getPackagesAPI } from "@navigraph/packages"

const config: NavigraphApp = {
  clientId: process.env.CLIENT_ID,
  clientSecret: process.env.CLIENT_SECRET,
  scopes: [Scope.FMSDATA],
}

if (!config.clientId || config.clientId.includes("<")) {
  alert("Please add your client credentials in lib/navigraph.ts.")
}

initializeApp(config)

// Wait 1s before accessing datastorage
// This is a potential workaround for the issue where datastorage does not deliver credentials on startup.
const dataStoreInit = new Promise(resolve => setTimeout(resolve, 1000))

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
