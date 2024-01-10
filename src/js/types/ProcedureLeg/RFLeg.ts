import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface RFLegData extends ProcedureLegBase {
  legType: LegType.RF

  fix: Fix

  turnDirection: TurnDirection

  recommendedNavaid?: Fix

  theta?: Degrees

  magneticCourse?: Degrees

  length: NauticalMiles

  arcCenterFix: Fix

  arcRadius: NauticalMiles
}
