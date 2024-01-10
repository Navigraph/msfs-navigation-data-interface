import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface XRLegData extends ProcedureLegBase {
  legType: LegType.CR | LegType.VR

  turnDirection?: TurnDirection

  recommendedNavaid: Fix

  theta: Degrees

  rho: NauticalMiles

  magneticCourse: Degrees
}
