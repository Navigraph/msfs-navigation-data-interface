import { LegType, ProcedureLegBase, TurnDirection } from ".";
import { Degrees } from "../math";

export interface VMLegData extends ProcedureLegBase {
  leg_type: LegType.VM;

  turn_direction?: TurnDirection;

  course: Degrees;
}
