import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from "."
import { Degrees, Feet } from "../types"

export interface CALegData extends ProcedureLegBase {
  leg_type: LegType.CA

  turn_direction?: TurnDirection

  magnetic_course: Degrees

  altitude: {
    altitude1: Feet

    descriptor: AltitudeDescriptor.AtOrAboveAlt1
  }
}
