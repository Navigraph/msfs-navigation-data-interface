import { Coordinates } from "./types"

export enum FixType {
  Airport = "A",
  NdbNavaid = "N",
  RunwayThreshold = "R",
  GlsNavaid = "G",
  IlsNavaid = "I",
  VhfNavaid = "V",
  Waypoint = "W",
}

export interface Fix {
  fix_type: FixType
  ident: string
  icao_code: string
  location: Coordinates
  airport_ident?: string
}
