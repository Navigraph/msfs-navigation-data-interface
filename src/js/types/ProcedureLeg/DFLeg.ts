import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface DFLegData extends ProcedureLegBase {
  leg_type: LegType.DF

  fix: Fix

  turn_direction?: TurnDirection

  recommended_navaid?: Fix

  theta?: Degrees

  rho?: NauticalMiles
}
