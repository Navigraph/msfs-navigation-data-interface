use serde::Serialize;

use crate::{
    math::{Coordinates, Degrees, Feet},
    sql_structs, v2,
};

#[derive(Serialize)]
pub struct GlsNavaid {
    /// The Geographic region where this navaid is
    pub area_code: String,
    /// The identifier of the airport which this navaid serves
    pub airport_ident: String,
    /// The icao prefix of the region this navaid is in
    pub icao_code: String,
    /// The identifier of this navaid, such as `G03P` or `A34A`
    pub ident: String,
    /// The category of this navaid, Technically can be multiple values, but the database only contains `1` as the
    /// value for this field
    pub category: String,
    /// The channel of this navaid
    pub channel: f64,
    /// The identifier of the runway this navaid serves
    pub runway_ident: String,
    /// The magnetic bearing of the approach to this navaid
    pub magnetic_approach_bearing: Degrees,
    /// The location of this navaid
    pub location: Coordinates,
    /// The angle of the approach to this navaid
    pub approach_angle: Degrees,
    /// The magnetic variation at this navaid
    pub magnetic_variation: f64,
    /// The elevation of this navaid
    pub elevation: Feet,
}

impl From<sql_structs::Gls> for GlsNavaid {
    fn from(gls: sql_structs::Gls) -> Self {
        Self {
            area_code: gls.area_code,
            airport_ident: gls.airport_identifier,
            icao_code: gls.icao_code,
            ident: gls.gls_ref_path_identifier,
            category: gls.gls_category,
            runway_ident: gls.runway_identifier,
            channel: gls.gls_channel,
            magnetic_approach_bearing: gls.gls_approach_bearing,
            location: Coordinates {
                lat: gls.station_latitude,
                long: gls.station_longitude,
            },
            approach_angle: gls.gls_approach_slope,
            magnetic_variation: gls.magentic_variation,
            elevation: gls.station_elevation,
        }
    }
}

impl From<v2::sql_structs::Gls> for GlsNavaid {
    fn from(gls: v2::sql_structs::Gls) -> Self {
        Self {
            area_code: gls.area_code,
            airport_ident: gls.airport_identifier,
            icao_code: gls.icao_code,
            ident: gls.gls_ref_path_identifier,
            category: gls.gls_category,
            runway_ident: gls.runway_identifier,
            channel: gls.gls_channel,
            magnetic_approach_bearing: gls.gls_approach_bearing,
            location: Coordinates {
                lat: gls.station_latitude,
                long: gls.station_longitude,
            },
            approach_angle: gls.gls_approach_slope,
            magnetic_variation: gls.magnetic_variation,
            elevation: gls.station_elevation,
        }
    }
}
