import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from "."
import { Fix } from "../fix"
import { Degrees, Feet, Minutes, NauticalMiles } from "../types"

export interface HALegData extends ProcedureLegBase {
  legType: LegType.HA

  fix: Fix

  turnDirection: TurnDirection

  theta?: Degrees

  rho?: NauticalMiles

  magneticCourse: Degrees

  length?: NauticalMiles

  lengthTime?: Minutes

  altitude: {
    altitude1: Feet

    descriptor: AltitudeDescriptor.AtOrAboveAlt1
  }
}
