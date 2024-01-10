import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface XDLegData extends ProcedureLegBase {
  legType: LegType.CD | LegType.VD

  turnDirection?: TurnDirection

  recommendedNavaid: Fix

  magneticCourse: Degrees

  length: NauticalMiles
}
