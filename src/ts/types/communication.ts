import { Coordinates } from "./math";

export enum FrequencyUnits {
  High = "H",
  VeryHigh = "V",
  UltraHigh = "U",
  /** Communication channel for 8.33 kHz spacing */
  CommChannel = "C",
}

export enum CommunicationType {
  AreaControlCenter = "ACC",
  AirliftCommandPost = "ACP",
  AirToAir = "AIR",
  ApproachControl = "APP",
  ArrivalControl = "ARR",
  AutomaticSurfaceObservingSystem = "ASO",
  AutomaticTerminalInformationServices = "ATI",
  AirportWeatherInformationBroadcast = "AWI",
  AutomaticWeatherObservingBroadcast = "AWO",
  AerodromeWeatherInformationService = "AWS",
  ClearanceDelivery = "CLD",
  ClearancePreTaxi = "CPT",
  ControlArea = "CTA",
  Control = "CTL",
  DepartureControl = "DEP",
  Director = "DIR",
  EnrouteFlightAdvisoryService = "EFS",
  Emergency = "EMR",
  FlightServiceStation = "FSS",
  GroundCommOutlet = "GCO",
  GroundControl = "GND",
  GateControl = "GET",
  HelicopterFrequency = "HEL",
  Information = "INF",
  MilitaryFrequency = "MIL",
  Multicom = "MUL",
  Operations = "OPS",
  PilotActivatedLighting = "PAL",
  Radio = "RDO",
  Radar = "RDR",
  RemoteFlightServiceStation = "RFS",
  RampTaxiControl = "RMP",
  AirportRadarServiceArea = "RSA",
  /** Terminal Control Area (TCA) */
  Tca = "TCA",
  /** Terminal Control Area (TMA) */
  Tma = "TMA",
  Terminal = "TML",
  TerminalRadarServiceArea = "TRS",
  TranscriberWeatherBroadcast = "TWE",
  Tower = "TWR",
  UpperAreaControl = "UAC",
  Unicom = "UNI",
  Volmet = "VOL",
}

export interface Communication {
  area_code: string;
  communication_type: CommunicationType;
  airport_ident?: string;
  fir_rdo_ident?: string;
  frequency: number;
  frequency_units: FrequencyUnits;
  callsign?: string;
  name?: string;
  location: Coordinates;
  remote_facility?: string;
  remote_facility_icao_code?: string;
  sector_facility?: string;
  sector_facility_icao_code?: string;
  sectorization?: string;
}
