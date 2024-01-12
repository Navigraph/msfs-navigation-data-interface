import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Degrees } from "../types"

export interface VMLegData extends ProcedureLegBase {
  leg_type: LegType.VM

  turn_direction?: TurnDirection

  magnetic_course: Degrees
}
