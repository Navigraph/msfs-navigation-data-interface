import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface XDLegData extends ProcedureLegBase {
  leg_type: LegType.CD | LegType.VD

  turn_direction?: TurnDirection

  recommended_navaid: Fix

  magnetic_course: Degrees

  length: NauticalMiles
}
