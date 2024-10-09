use serde::Serialize;

use crate::{
    math::{Coordinates, Degrees, Feet},
    sql_structs,
};

#[derive(Serialize, Clone)]
pub struct RunwayThreshold {
    /// The identifier of this runway, such as `RW18L` or `RW36R`
    pub ident: String,
    /// The icao prefix of the region that this runway is in.
    pub icao_code: String,
    /// The length of this runway in feet
    pub length: Feet,
    /// The width of this runway in feet
    pub width: Feet,
    /// The true bearing of this runway in degrees
    pub true_bearing: Degrees,
    /// The magnetic bearing of this runway in degrees.
    ///
    /// This field is rounded to the nearest degree
    pub magnetic_bearing: Degrees,
    /// The gradient of this runway in degrees
    pub gradient: Degrees,
    /// The geographic location of the landing threshold of this runway
    pub location: Coordinates,
    /// The elevation of the landing threshold of this runway in feet
    pub elevation: Feet,
    pub id: String,
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
            id: runway.id,
        }
    }
}
