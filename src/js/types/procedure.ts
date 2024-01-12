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
