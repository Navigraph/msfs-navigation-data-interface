import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from "."
import { Degrees, Feet } from "../math"

export interface VALegData extends ProcedureLegBase {
  leg_type: LegType.VA

  turn_direction?: TurnDirection

  course: Degrees

  altitude: {
    altitude1: Feet

    descriptor: AltitudeDescriptor.AtOrAboveAlt1
  }
}
