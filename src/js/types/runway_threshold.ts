import { Coordinates, Degrees, Feet } from "./math"

// Im not sure why we chose an enum, but I think its because its Y/N in the DFDv2 Spec
export enum RunwayLights {
  Yes = "Y",
  No = "N",
}

export enum RunwaySurface {
  Asphalt = "ASPH",
  Turf = "TURF",
  Gravel = "GRVL",
  Concrete = "CONC",
  Water = "WATE",
  Bitumen = "BITU",
  Unpaved = "UNPV",
}

export enum TrafficPattern {
  Left = "L",
  Right = "R",
}

export interface RunwayThreshold {
  ident: string
  icao_code: string
  length: Feet
  width: Feet
  true_bearing: Degrees
  magnetic_bearing: Degrees
  lights?: RunwayLights
  gradient: Degrees
  location: Coordinates
  elevation: Feet
  surface?: RunwaySurface
  traffic_pattern?: TrafficPattern
}
