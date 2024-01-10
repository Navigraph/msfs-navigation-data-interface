import { Degrees, Feet, Knots, NauticalMiles } from "../types"
import { AFLegData } from "./AFLeg"
import { CALegData } from "./CALeg"
import { CFLegData } from "./CFLeg"
import { DFLegData } from "./DFLeg"
import { FALegData } from "./FALeg"
import { FDLegData } from "./FDLeg"
import { FMLegData } from "./FMLeg"
import { HALegData } from "./HALeg"
import { HFLegData } from "./HFLeg"
import { HMLegData } from "./HMLeg"
import { IFLegData } from "./IFLeg"
import { RFLegData } from "./RFLeg"
import { TFLegData } from "./TFLeg"
import { VALegData } from "./VALeg"
import { VMLegData } from "./VMLeg"
import { XDLegData } from "./XDLeg"
import { XILegData } from "./XILeg"
import { XRLegData } from "./XRLeg"

export enum LegType {
  AF = "AF",
  CA = "CA",
  CD = "CD",
  CF = "CF",
  CI = "CI",
  CR = "CR",
  DF = "DF",
  FA = "FA",
  FC = "FC",
  FD = "FD",
  FM = "FM",
  HA = "HA",
  HF = "HF",
  HM = "HM",
  IF = "IF",
  PI = "PI",
  RF = "RF",
  TF = "TF",
  VA = "VA",
  VD = "VD",
  VI = "VI",
  VM = "VM",
  VR = "VR",
}

export enum TurnDirection {
  Left = "L",
  Right = "R",
}

export enum AltitudeDescriptor {
  AtAlt1 = "@",
  AtOrAboveAlt1 = "+",
  AtOrBelowAlt1 = "-",
  BetweenAlt1Alt2 = "B",
  AtOrAboveAlt2 = "C",
  AtAlt1GsMslAlt2 = "G",
  AtOrAboveAlt1GsMslAlt2 = "H",
  AtAlt1GsInterceptAlt2 = "I",
  AtOrAboveAlt1GsInterceptAlt2 = "J",
  AtOrAboveAlt1AngleAlt2 = "V",
  AtAlt1AngleAlt2 = "X",
  AtOrBelowAlt1AngleAlt2 = "Y",
}

export enum SpeedDescriptor {
  Mandatory = "@",
  Minimum = "+",
  Maximum = "-",
}

export type AltitudeConstraint =
  | {
      altitude1: Feet
      altitude2?: Feet
      descriptor: AltitudeDescriptor.AtAlt1 | AltitudeDescriptor.AtOrAboveAlt1 | AltitudeDescriptor.AtOrBelowAlt1
    }
  | {
      altitude1: Feet
      altitude2: Feet
      descriptor:
        | AltitudeDescriptor.BetweenAlt1Alt2
        | AltitudeDescriptor.AtOrAboveAlt2
        | AltitudeDescriptor.AtAlt1GsMslAlt2
        | AltitudeDescriptor.AtOrAboveAlt1GsMslAlt2
        | AltitudeDescriptor.AtAlt1GsInterceptAlt2
        | AltitudeDescriptor.AtOrAboveAlt1GsInterceptAlt2
        | AltitudeDescriptor.AtOrAboveAlt1AngleAlt2
        | AltitudeDescriptor.AtAlt1AngleAlt2
        | AltitudeDescriptor.AtOrBelowAlt1AngleAlt2
    }

export interface SpeedConstraint {
  value: Knots
  descriptor: SpeedDescriptor
}

export interface ProcedureLegBase {
  overfly: boolean

  altitude?: AltitudeConstraint

  speed?: SpeedConstraint

  verticalAngle?: Degrees

  rnp?: NauticalMiles
}

export type HXLegData = HALegData | HFLegData | HMLegData
export type XFLegData = AFLegData | CFLegData | DFLegData | IFLegData | RFLegData | TFLegData | HXLegData
export type FXLegData = FALegData | FMLegData | FDLegData
export type ProcedureLeg = XFLegData | CALegData | XILegData | XDLegData | VALegData | VMLegData | XRLegData
