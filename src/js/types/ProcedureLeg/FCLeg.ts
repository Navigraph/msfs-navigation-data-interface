import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../math"

export interface FCLegData extends ProcedureLegBase {
  leg_type: LegType.FC

  fix: Fix

  recommended_navaid: Fix

  turn_direction?: TurnDirection

  theta: Degrees

  rho: NauticalMiles

  course: Degrees

  length: NauticalMiles
}
