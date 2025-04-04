import { Coordinates, Degrees, MegaHertz, NauticalMiles } from "./math";

export interface VhfNavaid {
  area_code: string;
  airport_ident?: string;
  continent?: string;
  country?: string;
  datum_code?: string;
  icao_code: string;
  ident: string;
  name: string;
  frequency: MegaHertz;
  location: Coordinates;
  magnetic_variation?: Degrees;
  station_declination?: Degrees;
  range?: NauticalMiles;
}
