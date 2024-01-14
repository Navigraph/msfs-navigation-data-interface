use serde::Serialize;

use crate::{
    math::{Coordinates, Degrees, MegaHertz},
    sql_structs,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct VhfNavaid {
    /// Represents the geographic region in which this VhfNavaid is located
    pub area_code: String,
    /// The identifier of the airport that this VhfNavaid is associated with, if any
    pub airport_ident: Option<String>,
    /// The icao prefix of the region that this VhfNavaid is in.
    pub icao_code: String,
    /// The identifier of the VOR station used in this VhfNavaid (not unique), such as `ITA` or `NZ`
    pub ident: String,
    /// The formal name of the VOR station used in this VhfNavaid such as `NARSARSUAQ` or `PHOENIX MCMURDO STATION`
    pub name: String,
    /// The frequency of this the VOR station used in this `VhfNavaid` in megahertz
    pub frequency: MegaHertz,
    /// The geographic location of the VOR station used in this `VhfNavaid`
    pub location: Coordinates,
    /// The magnetic declination of this `VhfNavaid` in degrees
    pub station_declination: Option<Degrees>,
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
