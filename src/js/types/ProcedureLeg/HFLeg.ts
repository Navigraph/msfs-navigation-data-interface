import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, Minutes, NauticalMiles } from "../types"

export interface HFLegData extends ProcedureLegBase {
  legType: LegType.HF

  fix: Fix

  turnDirection: TurnDirection

  theta?: Degrees

  rho?: NauticalMiles

  magneticCourse: Degrees

  length?: NauticalMiles

  lengthTime?: Minutes
}
