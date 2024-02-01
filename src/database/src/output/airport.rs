use serde::Serialize;

use crate::{
    enums::{IfrCapability, RunwaySurfaceCode},
    math::{Coordinates, Feet},
    sql_structs,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct Airport {
    /// The unique identifier of the airport, such as `KLAX` or `EGLL`
    pub ident: String,
    /// Represents the geographic region of the world where this airport is located.
    pub area_code: String,
    /// Represents the icao prefix of the region that this airport is in.
    ///
    /// For most airports, this will be the same as the first two letters of the `ident`, such as `EG` for `EGLL`, or
    /// `LF` for `LFPG`.
    ///
    /// The notable exceptions to this are airports in the US, Canada, and Australia.
    pub icao_code: String,
    /// The geographic location of the airport's reference point
    pub location: Coordinates,
    /// The formal name of the airport such as `KENNEDY INTL` for `KJFK` or `HEATHROW` for `EGLL`
    pub name: String,
    pub ifr_capability: IfrCapability,
    /// The surface type of the longest runway at this airport.
    pub longest_runway_surface_code: Option<RunwaySurfaceCode>,
    /// The elevation in feet of the airport's reference point
    pub elevation: Feet,
    /// The altitude in feet where aircraft transition from `QNH/QFE` to `STD` barometer settings
    ///
    /// This field will usually be smaller than `transition_level` to define the lower bound of the transition band
    pub transition_altitude: Option<Feet>,
    /// The flight level in feet where aircraft transition from `QNH/QFE` to `STD` barometer settings
    ///
    /// This field will usually be larger than `transition_altitude` to define the upper bound of the transition band
    pub transition_level: Option<Feet>,
    /// The speed limit in knots that aircraft should not exceed while they are below `speed_limit_altitude` around
    /// this airport
    pub speed_limit: Option<Feet>,
    /// The altitude in feet that aircraft below which must stay below the `speed_limit` of this airport while nearby.
    pub speed_limit_altitude: Option<Feet>,
    /// The IATA identifier of this airport, such as `LHR` for `EGLL` or `JFK` for `KJFK`
    pub iata_ident: Option<String>,
}

impl From<sql_structs::Airports> for Airport {
    fn from(airport: sql_structs::Airports) -> Self {
        Self {
            ident: airport.airport_identifier,
            area_code: airport.area_code,
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
