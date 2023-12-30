use serde::Serialize;

use super::enums::{IfrCapability, RunwaySurfaceCode};

#[derive(Serialize)]
pub struct Coordinates {
    pub lat: f64,
    pub long: f64,
}

#[derive(Serialize)]
pub struct Airport {
    pub ident: String,
    pub icao_code: String,
    pub location: Coordinates,
    pub name: String,
    pub ifr_capability: IfrCapability,
    pub longest_runway_surface_code: Option<RunwaySurfaceCode>,
    pub elevation: f64,
    pub transition_altitude: Option<f64>,
    pub transition_level: Option<f64>,
    pub speed_limit: Option<f64>,
    pub speed_limit_altitude: Option<f64>,
    pub iata_ident: Option<String>,
}
