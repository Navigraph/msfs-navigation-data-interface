use serde::Serialize;

use crate::{
    math::{Coordinates, Degrees, MegaHertz},
    sql_structs,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct VhfNavaid {
    area_code: String,
    airport_ident: Option<String>,
    icao_code: String,
    ident: String,
    name: String,
    frequency: MegaHertz,
    location: Coordinates,
    station_declination: Option<Degrees>,
}

impl From<sql_structs::VhfNavaids> for VhfNavaid {
    fn from(navaid: sql_structs::VhfNavaids) -> Self {
        Self {
            area_code: navaid.area_code,
            airport_ident: navaid.airport_identifier,
            icao_code: navaid.icao_code,
            ident: navaid.vor_identifier,
            name: navaid.vor_name,
            frequency: navaid.vor_frequency,
            location: Coordinates {
                lat: navaid.vor_latitude,
                long: navaid.vor_longitude,
            },
            station_declination: navaid.station_declination,
        }
    }
}
