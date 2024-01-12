import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees } from "../types"

export interface XILegData extends ProcedureLegBase {
  leg_type: LegType.CI | LegType.VI

  turn_direction?: TurnDirection

  recommended_navaid?: Fix

  magnetic_course: Degrees
}
