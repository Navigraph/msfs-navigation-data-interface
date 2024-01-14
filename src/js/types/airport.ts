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
  area_code: string
  ident: string
  icao_code: string
  location: Coordinates
  name: string
  ifr_capability: IfrCapability
  longest_runway_surface_code: RunwaySurfaceCode
  elevation: Feet
  transition_altitude?: Feet
  transition_level?: Feet
  speed_limit?: Knots
  speed_limit_altitude?: Feet
  iata_ident?: string
}
