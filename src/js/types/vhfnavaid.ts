import { Coordinates, Degrees, MegaHertz } from "./math"

export interface VhfNavaid {
  area_code: string
  airport_ident?: string
  icao_code: string
  ident: string
  name: string
  frequency: MegaHertz
  location: Coordinates
  station_declination?: Degrees
}
