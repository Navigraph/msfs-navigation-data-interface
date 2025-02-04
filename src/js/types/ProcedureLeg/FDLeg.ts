import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../math"

export interface FDLegData extends ProcedureLegBase {
  leg_type: LegType.FD

  fix: Fix

  turn_direction?: TurnDirection

  recommended_navaid: Fix

  theta?: Degrees

  rho: NauticalMiles

  course: Degrees

  length: NauticalMiles
}
