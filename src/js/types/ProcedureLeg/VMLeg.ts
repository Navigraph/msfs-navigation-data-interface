import { LegType, ProcedureLegBase, TurnDirection } from "."
import { Degrees } from "../types"

export interface VMLegData extends ProcedureLegBase {
  legType: LegType.VM

  turnDirection?: TurnDirection

  magneticCourse: Degrees
}
