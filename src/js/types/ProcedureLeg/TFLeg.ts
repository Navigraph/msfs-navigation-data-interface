import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, Minutes, NauticalMiles } from "../math"

export interface TFLegData extends ProcedureLegBase {
  leg_type: LegType.TF

  fix: Fix

  turn_direction?: TurnDirection

  recommended_navaid?: Fix

  theta?: Degrees

  rho?: NauticalMiles

  course?: Degrees

  length?: NauticalMiles

  length_time?: Minutes
}
