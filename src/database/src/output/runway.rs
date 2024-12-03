use serde::Serialize;

use crate::{
    enums::{RunwayLights, RunwaySurface, TrafficPattern},
    math::{Coordinates, Degrees, Feet},
    sql_structs, v2,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Default)]
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
    /// Whether or not the runway has lights (v2 only)
    pub lights: Option<RunwayLights>,
    /// Material that the runway is made out of (v2 only)
    pub surface: Option<RunwaySurface>,
    /// The traffic pattern of the runway (v2 only)
    pub traffic_pattern: Option<TrafficPattern>,
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
            ..Default::default()
        }
    }
}

impl From<v2::sql_structs::Runways> for RunwayThreshold {
    fn from(runway: v2::sql_structs::Runways) -> Self {
        Self {
            ident: runway.runway_identifier,
            icao_code: runway.icao_code.unwrap_or("UNK".to_string()),
            length: runway.runway_length,
            width: runway.runway_width,
            true_bearing: runway.runway_true_bearing.unwrap_or_default(),
            magnetic_bearing: runway.runway_magnetic_bearing.unwrap_or_default(),
            gradient: runway.runway_gradient.unwrap_or_default(),
            location: Coordinates {
                lat: runway.runway_latitude.unwrap_or_default(),
                long: runway.runway_longitude.unwrap_or_default(),
            },
            elevation: runway.landing_threshold_elevation,
            surface: runway.surface_code,
            traffic_pattern: runway.traffic_pattern,
            lights: runway.runway_lights,
        }
    }
}
