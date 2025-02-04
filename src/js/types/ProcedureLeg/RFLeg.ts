import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../math"

export interface RFLegData extends ProcedureLegBase {
  leg_type: LegType.RF

  fix: Fix

  turn_direction: TurnDirection

  recommended_navaid?: Fix

  theta?: Degrees

  course?: Degrees

  length: NauticalMiles

  arc_center_fix: Fix

  arc_radius: NauticalMiles
}
