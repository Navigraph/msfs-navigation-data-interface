import { Coordinates } from "./math"

export interface Gate {
  area_code: string
  icao_code: string
  ident: string
  location: Coordinates
  name: string
}
