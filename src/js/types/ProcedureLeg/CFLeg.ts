import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface CFLegData extends ProcedureLegBase {
  legType: LegType.CF

  fix: Fix

  recommendedNavaid: Fix

  turnDirection?: TurnDirection

  theta: Degrees

  rho: NauticalMiles

  magneticCourse: Degrees

  length: NauticalMiles
}
