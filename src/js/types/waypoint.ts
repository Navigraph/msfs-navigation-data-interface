import { Coordinates } from "./math"

export interface Waypoint {
  area_code: string
  airport_ident?: string
  icao_code: string
  ident: string
  name: string
  location: Coordinates
}
