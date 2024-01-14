import { Coordinates, KiloHertz } from "./types"

export interface NdbNavaid {
  area_code: string
  airport_ident?: string
  icao_code: string
  ident: string
  name: string
  frequency: KiloHertz
  location: Coordinates
}
