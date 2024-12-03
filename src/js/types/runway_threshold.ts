import { Coordinates, Degrees, Feet } from "./math"

export interface RunwayThreshold {
  ident: string
  icao_code: string
  length: Feet
  width: Feet
  true_bearing: Degrees
  magnetic_bearing: Degrees
  lights?: string
  gradient: Degrees
  location: Coordinates
  elevation: Feet
  surface?: string
  traffic_pattern?: string
}
