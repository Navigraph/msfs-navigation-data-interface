export interface Airport {
  area_code: string | null
  icao_code: string
  airport_identifier: string
  airport_identifier_3letter: string | null
  airport_name: string | null
  airport_ref_latitude: number | null
  airport_ref_longitude: number | null
  ifr_capability: string | null
  longest_runway_surface_code: string | null
  elevation: number | null
  transition_altitude: number | null
  transition_level: number | null
  speed_limit: number | null
  speed_limit_altitude: number | null
  iata_ata_designator: string | null
}
