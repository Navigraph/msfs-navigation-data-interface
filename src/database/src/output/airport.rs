use serde::Serialize;

use crate::{
    enums::{IfrCapability, RunwaySurfaceCode},
    math::Coordinates,
    sql_structs,
};

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

impl From<sql_structs::Airports> for Airport {
    fn from(airport: sql_structs::Airports) -> Self {
        Self {
            ident: airport.airport_identifier,
            icao_code: airport.icao_code,
            location: Coordinates {
                lat: airport.airport_ref_latitude,
                long: airport.airport_ref_longitude,
            },
            name: airport.airport_name,
            ifr_capability: airport.ifr_capability,
            longest_runway_surface_code: airport.longest_runway_surface_code,
            elevation: airport.elevation,
            transition_altitude: airport.transition_altitude,
            transition_level: airport.transition_level,
            speed_limit: airport.speed_limit,
            speed_limit_altitude: airport.speed_limit_altitude,
            iata_ident: airport.iata_ata_designator,
        }
    }
}
