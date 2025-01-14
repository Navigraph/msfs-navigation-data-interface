use serde::Serialize;

use crate::{
    math::{Coordinates, Degrees, MegaHertz, NauticalMiles},
    sql_structs,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct VhfNavaid {
    /// Represents the geographic region in which this VhfNavaid is located
    pub area_code: String,
    /// Contenent of the navaid (v2 only)
    pub continent: Option<String>,
    /// Country of the navaid (v2 only)
    pub country: Option<String>,
    /// 3 Letter identifier describing the local horizontal identifier (v2 only)
    pub datum_code: Option<String>,
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
    /// Magnetic variation
    pub magnetic_variation: Option<Degrees>,
    /// VOR range (v2 only)
    pub range: Option<NauticalMiles>,
}

impl From<sql_structs::VhfNavaids> for VhfNavaid {
    fn from(navaid: sql_structs::VhfNavaids) -> Self {
        Self {
            area_code: navaid.area_code,
            airport_ident: navaid.airport_identifier,
            // Not entirely sure if this is behaviour we intend
            icao_code: navaid.icao_code.unwrap_or_default(),
            ident: navaid.navaid_identifier,
            name: navaid.navaid_name,
            frequency: navaid.navaid_frequency,
            location: Coordinates {
                lat: navaid
                    .navaid_latitude
                    .unwrap_or(navaid.dme_latitude.unwrap_or_default()),
                long: navaid
                    .navaid_longitude
                    .unwrap_or(navaid.dme_longitude.unwrap_or_default()),
            },
            station_declination: navaid.station_declination,
            continent: navaid.continent,
            country: navaid.country,
            magnetic_variation: navaid.magnetic_variation,
            range: navaid.range,
            datum_code: navaid.datum_code,
        }
    }
}
