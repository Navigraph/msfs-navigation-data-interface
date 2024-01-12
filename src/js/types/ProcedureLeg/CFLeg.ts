import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface CFLegData extends ProcedureLegBase {
  leg_type: LegType.CF

  fix: Fix

  recommended_navaid: Fix

  turn_direction?: TurnDirection

  theta: Degrees

  rho: NauticalMiles

  magnetic_course: Degrees

  length: NauticalMiles
}
