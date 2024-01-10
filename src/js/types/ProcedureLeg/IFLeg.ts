import { LegType, ProcedureLegBase } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface IFLegData extends ProcedureLegBase {
  legType: LegType.IF

  fix: Fix

  recommendedNavaid?: Fix

  theta?: Degrees

  rho?: NauticalMiles
}
