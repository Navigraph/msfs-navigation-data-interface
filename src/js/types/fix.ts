import { Coordinates } from "./types"

export enum FixType {
  Airport,
  NdbNavaid,
  RunwayThreshold,
  GlsNavaid,
  IlsNavaid,
  VhfNavaid,
  Waypoint,
}

export interface Fix {
  fixType: FixType
  ident: string
  icaoCode: string
  location: Coordinates
  airportIdentifier?: string
}
