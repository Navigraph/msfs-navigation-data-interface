use sentry::capture_message;
use serde::Serialize;

use crate::database::utils::{Coordinates, Degrees, Feet};

use super::{
    enums::{RunwayLights, RunwaySurface, TrafficPattern},
    sql,
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
    /// Whether or not the runway has lights (v2 only)
    pub lights: Option<RunwayLights>,
    /// Material that the runway is made out of (v2 only)
    pub surface: Option<RunwaySurface>,
    /// The traffic pattern of the runway (v2 only)
    pub traffic_pattern: Option<TrafficPattern>,
}

impl From<sql::Runways> for RunwayThreshold {
    fn from(runway: sql::Runways) -> Self {
        let mut error_in_row = false;

        let runway_new = Self {
            ident: runway.runway_identifier.clone(),
            icao_code: runway.icao_code.unwrap_or_else(|| {
                error_in_row = true;
                "UNKN".to_string()
            }),
            length: runway.runway_length,
            width: runway.runway_width,
            true_bearing: runway.runway_true_bearing.unwrap_or_else(|| {
                error_in_row = true;
                0.
            }),
            magnetic_bearing: runway.runway_magnetic_bearing.unwrap_or_else(|| {
                error_in_row = true;
                0.
            }),
            gradient: runway.runway_gradient.unwrap_or_default(),
            location: Coordinates {
                lat: runway.runway_latitude.unwrap_or_else(|| {
                    error_in_row = true;
                    0.
                }),
                long: runway.runway_longitude.unwrap_or_else(|| {
                    error_in_row = true;
                    0.
                }),
            },
            elevation: runway.landing_threshold_elevation,
            surface: runway.surface_code,
            traffic_pattern: runway.traffic_pattern,
            lights: runway.runway_lights,
        };

        if error_in_row {
            let error_text = format!(
                "Error found in Runway: {}",
                serde_json::to_string(&runway_new).unwrap_or(format!(
                    "Error serializing output, {} runway {}",
                    runway.airport_identifier, runway.runway_identifier,
                ))
            );

            capture_message(&error_text, sentry::Level::Warning);
        }

        runway_new
    }
}
