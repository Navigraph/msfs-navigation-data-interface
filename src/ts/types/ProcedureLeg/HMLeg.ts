import { LegType, ProcedureLegBase, TurnDirection } from ".";
import { Fix } from "../fix";
import { Degrees, Minutes, NauticalMiles } from "../math";

export interface HMLegData extends ProcedureLegBase {
  leg_type: LegType.HM;

  fix: Fix;

  turn_direction: TurnDirection;

  theta?: Degrees;

  rho?: NauticalMiles;

  course: Degrees;

  length?: NauticalMiles;

  length_time?: Minutes;
}
