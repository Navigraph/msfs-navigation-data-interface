import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface FDLegData extends ProcedureLegBase {
  legType: LegType.FD

  fix: Fix

  turnDirection?: TurnDirection

  recommendedNavaid: Fix

  theta?: Degrees

  rho: NauticalMiles

  magneticCourse: Degrees

  length: NauticalMiles
}
