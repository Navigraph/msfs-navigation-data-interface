import { Fix } from "./fix";

export enum AirwayRouteType {
  Control = "C",
  DirectRoute = "D",
  HelicopterRoute = "H",
  OfficialDesignatedAirwaysExpectRnavAirways = "O",
  RnavAirways = "R",
  UndesignatedAtsRoute = "S",
}

export enum AirwayLevel {
  Both = "B",
  High = "H",
  Low = "L",
}

export enum AirwayDirection {
  Forward = "F",
  Backward = "B",
}

export interface Airway {
  ident: string;
  fixes: Fix[];
  route_type: AirwayRouteType;
  level: AirwayLevel;
  direction?: AirwayDirection;
}
