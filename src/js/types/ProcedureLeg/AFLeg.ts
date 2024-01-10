import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface AFLegData extends ProcedureLegBase {
  legType: LegType.AF

  fix: Fix

  turnDirection: TurnDirection

  recommendedNavaid: Fix

  theta: Degrees

  rho: NauticalMiles

  magneticCourse: Degrees
}
