import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, Feet, NauticalMiles } from "../math"

export interface FALegData extends ProcedureLegBase {
  leg_type: LegType.FA

  fix: Fix

  recommended_navaid: Fix

  turn_direction?: TurnDirection

  theta: Degrees

  rho: NauticalMiles

  magnetic_course: Degrees

  altitude: {
    altitude1: Feet

    descriptor: AltitudeDescriptor.AtOrAboveAlt1
  }
}
