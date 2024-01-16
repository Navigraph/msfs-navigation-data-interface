import { Coordinates, Degrees, Metres } from "./types"

export enum ApproachTypeIdentifier {
  LocalizerPerformanceVerticalGuidance = "LPV",
  LocalizerPerformance = "LP",
}

export interface PathPoint {
  area_code: string
  airport_ident: string
  icao_code: string
  /// The identifier of the approach this path point is used in, such as `R36RY` or `R20`
  approach_ident: string
  /// The identifier of the runway this path point is used with, such as `RW02` or `RW36L`
  runway_ident: string
  ident: string
  landing_threshold_location: Coordinates
  ltp_ellipsoid_height: Metres
  fpap_ellipsoid_height: Metres
  ltp_orthometric_height?: Metres
  fpap_orthometric_height?: Metres
  glidepath_angle: Degrees
  flightpath_alignment_location: Coordinates
  course_width: Metres
  length_offset: Metres
  path_point_tch: Metres
  horizontal_alert_limit: Metres
  vertical_alert_limit: Metres
  gnss_channel_number: number
  approach_type: ApproachTypeIdentifier
}
