import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface DFLegData extends ProcedureLegBase {
  legType: LegType.DF

  fix: Fix

  turnDirection?: TurnDirection

  recommendedNavaid?: Fix

  theta?: Degrees

  rho?: NauticalMiles
}
