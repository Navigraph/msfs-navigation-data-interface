import { Coordinates, Feet, Knots } from "./types"

export enum IfrCapability {
  Yes = "Y",
  No = "N",
}

export enum RunwaySurfaceCode {
  Hard = "H",
  Soft = "S",
  Water = "W",
  Unknown = "U",
}

export interface Airport {
  ident: string
  icaoCode: string
  location: Coordinates
  areaCode: string
  name: string
  ifrCapability: IfrCapability
  longestRunwaySurfaceCode: RunwaySurfaceCode
  elevation: Feet
  transitionAltitude?: Feet
  transitionLevel?: Feet
  speedLimit?: Knots
  speedLimitAltitude?: Feet
  iataIdent?: string
}
