import { TurnDirection } from "./ProcedureLeg"
import { Coordinates, Degrees, NauticalMiles } from "./math"

export enum ControlledAirspaceType {
  ClassC = "A",
  ControlArea = "C",
  TmaOrTca = "K",
  IcaoTerminalControlArea = "M",
  MilitaryControlZone = "Q",
  RadarZone = "R",
  ClassB = "T",
  TerminalControlArea = "W",
  TerminalArea = "X",
  TerminalRadarServiceArea = "Y",
  ClassD = "Z",
}

export enum RestrictiveAirspaceType {
  Alert = "A",
  Caution = "C",
  Danger = "D",
  Military = "M",
  Prohibited = "P",
  Restricted = "R",
  Training = "T",
  Warning = "W",
  Unknown = "U",
}

export enum PathType {
  Circle = "C",
  GreatCircle = "G",
  RhumbLine = "R",
  Arc = "A",
}

export interface Arc {
  origin: Coordinates
  distance: NauticalMiles
  bearing: Degrees
  direction: TurnDirection
}

export interface Path {
  location: Coordinates
  arc?: Arc
  path_type: PathType
}

export interface ControlledAirspace {
  area_code: string
  icao_code: string
  airspace_center: string
  name: string
  airspace_type: ControlledAirspaceType
  boundary_paths: Path[]
}

export interface RestrictiveAirspace {
  area_code: string
  icao_code: string
  designation: string
  name: string
  airspace_type: RestrictiveAirspaceType
  boundary_paths: Path[]
}
