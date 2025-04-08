import { Coordinates, Degrees, Feet } from "./math";

// Im not sure why we chose an enum, but I think its because its Y/N in the DFDv2 Spec
export enum RunwayLights {
  Yes = "Y",
  No = "N",
}

export enum RunwaySurface {
  Gravel = "GRVL",
  Unpaved = "UNPV",
  Asphalt = "ASPH",
  Turf = "TURF",
  Dirt = "DIRT",
  Concrete = "CONC",
  Water = "WATE",
  Sand = "SAND",
  Coral = "CORL",
  Paved = "PAVD",
  Grass = "GRAS",
  Bitumen = "BITU",
  Planking = "PLNG",
  Clay = "CLAY",
  Ice = "ICE",
  Silt = "SILT",
  Laterite = "LATE",
  Tarmac = "TARM",
  Macadam = "MACA",
  Sealed = "SELD",
  Soil = "SOIL",
  Brick = "BRCK",
  Unknown = "UNKN",
  Mats = "MATS",
  Snow = "SNOW",
  Treated = "TRTD",
}

export enum TrafficPattern {
  Left = "L",
  Right = "R",
}

export interface RunwayThreshold {
  ident: string;
  icao_code: string;
  length: Feet;
  width: Feet;
  true_bearing: Degrees;
  magnetic_bearing: Degrees;
  lights?: RunwayLights;
  gradient: Degrees;
  location: Coordinates;
  elevation: Feet;
  surface?: RunwaySurface;
  traffic_pattern?: TrafficPattern;
}
