import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../math"

export interface AFLegData extends ProcedureLegBase {
  leg_type: LegType.AF

  fix: Fix

  turn_direction: TurnDirection

  recommended_navaid: Fix

  theta: Degrees

  rho: NauticalMiles

  course: Degrees
}
