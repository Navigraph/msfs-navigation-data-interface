use serde::Serialize;

use crate::{math::Coordinates, sql_structs};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct Waypoint {
    /// Represents the geographic region in which this Waypoint is located
    pub area_code: String,
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
}

impl From<sql_structs::Waypoints> for Waypoint {
    fn from(waypoint: sql_structs::Waypoints) -> Self {
        Self {
            area_code: waypoint.area_code,
            airport_ident: waypoint.region_code,
            icao_code: waypoint.icao_code,
            ident: waypoint.waypoint_identifier,
            name: waypoint.waypoint_name,
            location: Coordinates {
                lat: waypoint.waypoint_latitude,
                long: waypoint.waypoint_longitude,
            },
        }
    }
}
