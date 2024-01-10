import { ProcedureLeg } from "./ProcedureLeg"

export interface Transition {
  ident: string
  legs: ProcedureLeg[]
}

export interface Departure {
  ident: string
  runwayTransitions: Transition[]
  commonLegs: ProcedureLeg[]
  enrouteTransitions: Transition[]
  engineOutLegs: ProcedureLeg[]

  identicalRunwayTransitions: boolean
}
