import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../math"

export interface XRLegData extends ProcedureLegBase {
  leg_type: LegType.CR | LegType.VR

  turn_direction?: TurnDirection

  recommended_navaid: Fix

  theta: Degrees

  rho: NauticalMiles

  course: Degrees
}
