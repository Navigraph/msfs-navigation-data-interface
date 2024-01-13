use serde::Serialize;

use crate::{
    math::{Coordinates, Degrees, Feet},
    sql_structs,
};

#[derive(Serialize, Clone)]
pub struct RunwayThreshold {
    pub ident: String,
    pub icao_code: String,
    pub length: Feet,
    pub width: Feet,
    pub true_bearing: Degrees,
    pub magnetic_bearing: Degrees,
    pub gradient: Degrees,
    pub location: Coordinates,
    pub elevation: Feet,
}

impl From<sql_structs::Runways> for RunwayThreshold {
    fn from(runway: sql_structs::Runways) -> Self {
        Self {
            ident: runway.runway_identifier,
            icao_code: runway.icao_code,
            length: runway.runway_length,
            width: runway.runway_width,
            true_bearing: runway.runway_true_bearing,
            magnetic_bearing: runway.runway_magnetic_bearing,
            gradient: runway.runway_gradient,
            location: Coordinates {
                lat: runway.runway_latitude,
                long: runway.runway_longitude,
            },
            elevation: runway.landing_threshold_elevation,
        }
    }
}
