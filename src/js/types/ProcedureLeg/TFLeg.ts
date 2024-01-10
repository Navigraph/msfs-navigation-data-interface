import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, Minutes, NauticalMiles } from "../types"

export interface TFLegData extends ProcedureLegBase {
  legType: LegType.TF

  fix: Fix

  turnDirection?: TurnDirection

  recommendedNavaid?: Fix

  theta?: Degrees

  rho?: NauticalMiles

  magneticCourse?: Degrees

  length?: NauticalMiles

  lengthTime?: Minutes
}
