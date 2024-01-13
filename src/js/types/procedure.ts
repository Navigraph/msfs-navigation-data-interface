import { ProcedureLeg } from "./ProcedureLeg"

export interface Transition {
  ident: string
  legs: ProcedureLeg[]
}

export interface Departure {
  ident: string
  runway_transitions: Transition[]
  common_legs: ProcedureLeg[]
  enroute_transitions: Transition[]
  engine_out_legs: ProcedureLeg[]

  identical_runway_transitions: boolean
}

export interface Arrival {
  ident: string
  enroute_transitions: Transition[]
  common_legs: ProcedureLeg[]
  runway_transitions: Transition[]

  identical_runway_transitions: boolean
}

export enum ApproachType {
  LocBackcourse = "B",
  VorDme = "D",
  Fms = "F",
  Igs = "G",
  Ils = "I",
  Gls = "J",
  Loc = "L",
  Mls = "M",
  Ndb = "N",
  Gps = "P",
  NdbDme = "Q",
  Rnav = "R",
  Vortac = "S",
  Tacan = "T",
  Sdf = "U",
  Vor = "V",
  MlsTypeA = "W",
  Lda = "X",
  MlsTypeBC = "Y",
}

export interface Approach {
  ident: string
  transitions: Transition[]
  legs: ProcedureLeg[]
  missed_legs: ProcedureLeg[]

  runway_ident: string
  approach_type: ApproachType
}
