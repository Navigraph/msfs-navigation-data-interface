import { Coordinates, Degrees, Feet } from "./math"

export interface RunwayThreshold {
  ident: string
  icao_code: string
  length: Feet
  width: Feet
  true_bearing: Degrees
  magnetic_bearing: Degrees
  gradient: Degrees
  location: Coordinates
  elevation: Feet
}
