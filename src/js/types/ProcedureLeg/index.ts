import { Degrees, Feet, Knots, NauticalMiles } from "../math"
import { AFLegData } from "./AFLeg"
import { CALegData } from "./CALeg"
import { CFLegData } from "./CFLeg"
import { DFLegData } from "./DFLeg"
import { FALegData } from "./FALeg"
import { FCLegData } from "./FCLeg"
import { FDLegData } from "./FDLeg"
import { FMLegData } from "./FMLeg"
import { HALegData } from "./HALeg"
import { HFLegData } from "./HFLeg"
import { HMLegData } from "./HMLeg"
import { IFLegData } from "./IFLeg"
import { PILegData } from "./PILeg"
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

export enum RequiresAuthentication {
  Authorized = "Y",
  NotAuthorized = "N",
}

export enum GnssFmsIndication {
  NotAuthorized = "0",
  GnssMonitored = "1",
  GnssNotMonitored = "2",
  GnssAuthorized = "3",
  FmsAuthorized = "4",
  GnssFmsAuthorized = "5",
  RnavSbasAuthorized = "A",
  RnavSbasNotAuthorized = "B",
  RnavSbasUnspecified = "C",
  GpsProcedure = "D",
  Unspecified = "U",
}

export interface ProcedureAuthorization {
  authorized: Authorized
  name: string
}

export enum Authorized {
  Authorized = "A",
  NotAuthorized = "N",
}

export interface ProcedureLegBase {
  overfly: boolean

  altitude?: AltitudeConstraint

  speed?: SpeedConstraint

  vertical_angle?: Degrees

  rnp?: NauticalMiles

  // I'm not sure what types of legs these are in so it'll be here until I have more info
  ra?: RequiresAuthentication

  gnss_fms_indication?: GnssFmsIndication

  lnav_authorized?: ProcedureAuthorization

  lnav_vnav_authorized?: ProcedureAuthorization
}

export type HXLegData = HALegData | HFLegData | HMLegData
export type XFLegData = AFLegData | CFLegData | DFLegData | IFLegData | RFLegData | TFLegData | HXLegData
export type FXLegData = FALegData | FCLegData | FMLegData | FDLegData
export type ProcedureLeg =
  | XFLegData
  | FXLegData
  | CALegData
  | XILegData
  | XDLegData
  | VALegData
  | VMLegData
  | XRLegData
  | PILegData

export * from "./AFLeg"
export * from "./CALeg"
export * from "./CFLeg"
export * from "./DFLeg"
export * from "./FALeg"
export * from "./FCLeg"
export * from "./FDLeg"
export * from "./FMLeg"
export * from "./HALeg"
export * from "./HFLeg"
export * from "./HMLeg"
export * from "./IFLeg"
export * from "./PILeg"
export * from "./RFLeg"
export * from "./TFLeg"
export * from "./VALeg"
export * from "./VMLeg"
export * from "./XDLeg"
export * from "./XILeg"
export * from "./XRLeg"
