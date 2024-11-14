use serde::Serialize;

use crate::{
    enums::{IfrCapability, RunwaySurfaceCode},
    math::{Coordinates, Degrees, Feet},
    sql_structs,
    v2,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
pub struct Airport {
    /// The unique identifier of the airport, such as `KLAX` or `EGLL`
    pub ident: String,
    /// Represents the geographic region of the world where this airport is located.
    pub area_code: String,
    /// Represents the icao prefix of the region that this airport is in.
    ///
    /// For most airports, this will be the same as the first two letters of the `ident`, such as `EG` for `EGLL`, or
    /// `LF` for `LFPG`.
    /// Airport type (see Appendix 3.38) (v2 only)
    /// The notable exceptions to this are airports in the US, Canada, and Australia.
    pub icao_code: String,
    pub airport_type: Option<String>,
    /// The geographic location of the airport's reference point
    pub location: Coordinates,
    /// The airport's general area (v2 only)
    pub city: Option<String>,
    pub continent: Option<String>,
    pub country: Option<String>,
    pub country_3letter: Option<String>,
    pub state: Option<String>,
    pub state_2letter: Option<String>,
    /// The formal name of the airport such as `KENNEDY INTL` for `KJFK` or `HEATHROW` for `EGLL`
    pub name: String,
    pub ifr_capability: IfrCapability,
    /// The surface type of the longest runway at this airport.
    pub longest_runway_surface_code: Option<RunwaySurfaceCode>,
    /// The elevation in feet of the airport's reference point
    pub elevation: Feet,
    /// Magnetic north in Degrees (v2 only)
    pub magnetic_variation: Option<Degrees>,
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
            ..Default::default()
        }
    }
}

impl From<v2::sql_structs::Airports> for Airport {
    fn from(airport: v2::sql_structs::Airports) -> Self {
        Self {
            ident: airport.airport_identifier,
            name: airport.airport_name,
            location: Coordinates {
                lat: airport.airport_ref_latitude,
                long: airport.airport_ref_longitude,
            },
            airport_type: Some(airport.airport_type),
            area_code: airport.area_code,
            iata_ident: airport.ata_iata_code,
            city: airport.city,
            continent: airport.continent,
            country: airport.country,
            country_3letter: airport.country_3letter,
            elevation: airport.elevation,
            icao_code: airport.icao_code,
            ifr_capability: airport.ifr_capability.unwrap_or(IfrCapability::No),
            longest_runway_surface_code: Some(airport.longest_runway_surface_code),
            magnetic_variation: airport.magnetic_variation,
            transition_altitude: airport.transition_altitude,
            transition_level: airport.transition_level,
            speed_limit: airport.speed_limit,
            speed_limit_altitude: airport.speed_limit_altitude.and_then(|val| val.parse::<f64>().ok()),
            state: airport.state,
            state_2letter: airport.state_2letter,
        }
    }
}
