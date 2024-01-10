import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from "."
import { Degrees, Feet } from "../types"

export interface VALegData extends ProcedureLegBase {
  legType: LegType.VA

  turnDirection?: TurnDirection

  magneticCourse: Degrees

  altitude: {
    altitude1: Feet

    descriptor: AltitudeDescriptor.AtOrAboveAlt1
  }
}
