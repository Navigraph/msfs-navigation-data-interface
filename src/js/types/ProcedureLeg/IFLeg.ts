import { LegType, ProcedureLegBase } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../math"

export interface IFLegData extends ProcedureLegBase {
  leg_type: LegType.IF

  fix: Fix

  recommended_navaid?: Fix

  theta?: Degrees

  rho?: NauticalMiles
}
