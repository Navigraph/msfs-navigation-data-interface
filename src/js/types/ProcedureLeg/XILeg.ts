import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees } from "../types"

export interface XILegData extends ProcedureLegBase {
  legType: LegType.CI | LegType.VI

  turnDirection?: TurnDirection

  recommendedNavaid?: Fix

  magneticCourse: Degrees
}
