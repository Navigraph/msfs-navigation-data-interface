import { AltitudeDescriptor, LegType, ProcedureLegBase, TurnDirection } from ".";
import { Fix } from "../fix";
import { Degrees, Feet, Minutes, NauticalMiles } from "../math";

export interface HALegData extends ProcedureLegBase {
  leg_type: LegType.HA;

  fix: Fix;

  turn_direction: TurnDirection;

  theta?: Degrees;

  rho?: NauticalMiles;

  course: Degrees;

  length?: NauticalMiles;

  length_time?: Minutes;

  altitude: {
    altitude1: Feet;

    descriptor: AltitudeDescriptor.AtOrAboveAlt1;
  };
}
