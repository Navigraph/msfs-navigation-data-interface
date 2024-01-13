use serde::Serialize;

use crate::{math::Coordinates, sql_structs};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct Waypoint {
    area_code: String,
    airport_ident: Option<String>,
    icao_code: String,
    ident: String,
    name: String,
    location: Coordinates,
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
