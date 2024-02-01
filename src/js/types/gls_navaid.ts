import { Coordinates, Degrees, Feet } from "./math"

export interface GlsNavaid {
  /** The Geographic region where this navaid is */
  area_code: string
  /** The identifier of the airport which this navaid serves */
  airport_ident: string
  /** The icao prefix of the region this navaid is in */
  icao_code: string
  /** The identifier of this navaid, such as `G03P` or `A34A` */
  ident: string
  /** The category of this navaid, Technically can be multiple values, but the database only contains `1` as the
   value for this field */
  category: string
  /** The channel of this navaid */
  channel: number
  /** The identifier of the runway this navaid serves */
  runway_ident: string
  /** The magnetic bearing of the approach to this navaid */
  magnetic_approach_bearing: Degrees
  /** The location of this navaid */
  location: Coordinates
  /** The angle of the approach to this navaid */
  approach_angle: Degrees
  /** The magnetic variation at this navaid */
  magnetic_variation: number
  /** The elevation of this navaid */
  elevation: Feet
}
