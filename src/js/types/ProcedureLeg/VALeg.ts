import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from "."
import { Degrees, Feet } from "../types"

export interface VALegData extends ProcedureLegBase {
  leg_type: LegType.VA

  turn_direction?: TurnDirection

  magnetic_course: Degrees

  altitude: {
    altitude1: Feet

    descriptor: AltitudeDescriptor.AtOrAboveAlt1
  }
}
