import { Coordinates, Degrees, Feet, Knots } from "./math";

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
  airport_type?: string;
  area_code: string;
  ident: string;
  icao_code: string;
  city?: string;
  continent?: string;
  country?: string;
  country_3letter?: string;
  state?: string;
  state_2letter?: string;
  location: Coordinates;
  name: string;
  ifr_capability: IfrCapability;
  longest_runway_surface_code: RunwaySurfaceCode;
  elevation: Feet;
  transition_altitude?: Feet;
  transition_level?: Feet;
  speed_limit?: Knots;
  speed_limit_altitude?: Feet;
  iata_ident?: string;
  magnetic_variation?: Degrees;
}
