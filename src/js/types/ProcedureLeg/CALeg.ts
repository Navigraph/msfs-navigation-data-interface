import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from "."
import { Degrees, Feet } from "../types"

export interface CALegData extends ProcedureLegBase {
  legType: LegType.CA

  turnDirection?: TurnDirection

  magneticCourse: Degrees

  altitude: {
    altitude1: Feet

    descriptor: AltitudeDescriptor.AtOrAboveAlt1
  }
}
