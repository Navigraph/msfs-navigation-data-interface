use serde::Serialize;

use crate::database::utils::{Coordinates, Degrees};

use super::sql;

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct Waypoint {
    /// Represents the geographic region in which this Waypoint is located
    pub area_code: String,
    /// Contenent of the waypoint (v2 only)
    pub continent: Option<String>,
    /// Country of the waypoint (v2 only)
    pub country: Option<String>,
    /// 3 Letter identifier describing the local horizontal identifier (v2 only)
    pub datum_code: Option<String>,
    /// The identifier of the airport that this Waypoint is associated with, if any
    pub airport_ident: Option<String>,
    /// The icao prefix of the region that this Waypoint is in.
    pub icao_code: String,
    /// The identifier of this Waypoint (not unique), such as `IRNMN` or `BRAIN`
    pub ident: String,
    /// The formal name of this Waypoint such as `HJALTEYRI AKUREYRI` or `ORAN`
    pub name: String,
    /// The geographic location of this Waypoint
    pub location: Coordinates,
    /// Magnetic variation (v2 only)
    pub magnetic_variation: Option<Degrees>,
}

impl From<sql::Waypoints> for Waypoint {
    fn from(waypoint: sql::Waypoints) -> Self {
        let mut error_in_row = false;

        Self {
            area_code: waypoint.area_code,
            airport_ident: waypoint.region_code,
            // Not entirely sure if this is behaviour we intend
            icao_code: waypoint.icao_code.unwrap_or_else(|| {
                error_in_row = true;
                "UNKN".to_string()
            }),
            ident: waypoint.waypoint_identifier,
            name: waypoint.waypoint_name,
            location: Coordinates {
                lat: waypoint.waypoint_latitude,
                long: waypoint.waypoint_longitude,
            },
            continent: waypoint.continent,
            country: waypoint.country,
            magnetic_variation: waypoint.magnetic_varation,
            datum_code: waypoint.datum_code,
        }
    }
}
