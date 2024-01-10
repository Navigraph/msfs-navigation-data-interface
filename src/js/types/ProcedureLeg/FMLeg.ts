import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, NauticalMiles } from "../types"

export interface FMLegData extends ProcedureLegBase {
  legType: LegType.FM

  fix: Fix

  recommendedNavaid: Fix

  turnDirection?: TurnDirection

  theta: Degrees

  rho: NauticalMiles

  magneticCourse: Degrees
}
