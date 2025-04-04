import { Coordinates, KiloHertz, NauticalMiles } from "./math";

export interface NdbNavaid {
  area_code: string;
  continent?: string;
  country?: string;
  datum_code?: string;
  airport_ident?: string;
  icao_code: string;
  ident: string;
  name: string;
  frequency: KiloHertz;
  location: Coordinates;
  range?: NauticalMiles;
}
