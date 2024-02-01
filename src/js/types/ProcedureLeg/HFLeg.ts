import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, Minutes, NauticalMiles } from "../math"

export interface HFLegData extends ProcedureLegBase {
  leg_type: LegType.HF

  fix: Fix

  turn_direction: TurnDirection

  theta?: Degrees

  rho?: NauticalMiles

  magnetic_course: Degrees

  length?: NauticalMiles

  length_time?: Minutes
}
