import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, Feet, NauticalMiles } from "../types"

export interface FALegData extends ProcedureLegBase {
  legType: LegType.FA

  fix: Fix

  recommendedNavaid: Fix

  turnDirection?: TurnDirection

  theta: Degrees

  rho: NauticalMiles

  magneticCourse: Degrees

  altitude: {
    altitude1: Feet

    descriptor: AltitudeDescriptor.AtOrAboveAlt1
  }
}
