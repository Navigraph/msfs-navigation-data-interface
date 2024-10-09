use serde::Serialize;

use crate::{
    math::{Coordinates, KiloHertz},
    sql_structs, v2,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct NdbNavaid {
    /// Represents the geographic region in which this NdbNavaid is located
    pub area_code: String,
    /// The identifier of the airport that this NdbNavaid is associated with, if any
    pub airport_ident: Option<String>,
    /// The icao prefix of the region that this NdbNavaid is in.
    pub icao_code: String,
    /// The identifier of this NdbNavaid (not unique), such as `BI` or `PHH`
    pub ident: String,
    /// The formal name of this NdbNavaid such as `HERBB OLATHE` or `KEDZI CHICAGO`
    pub name: String,
    /// The frequency of this NdbNavaid in kilohertz
    pub frequency: KiloHertz,
    /// The geographic location of thie NdbNavaid
    pub location: Coordinates,
    /// The id thats associated with the waypoint (used for some internal functions)
    pub id: String,
}

impl From<sql_structs::NdbNavaids> for NdbNavaid {
    fn from(navaid: sql_structs::NdbNavaids) -> Self {
        Self {
            area_code: navaid.area_code,
            airport_ident: navaid.airport_identifier,
            icao_code: navaid.icao_code,
            ident: navaid.ndb_identifier,
            name: navaid.ndb_name,
            frequency: navaid.ndb_frequency,
            location: Coordinates {
                lat: navaid.ndb_latitude,
                long: navaid.ndb_longitude,
            },
            id: navaid.id,
        }
    }
}

impl From<v2::sql_structs::NdbNavaids> for NdbNavaid {
    fn from(navaid: v2::sql_structs::NdbNavaids) -> Self {
        Self {
            area_code: navaid.area_code,
            airport_ident: navaid.airport_identifier,
            icao_code: navaid.icao_code,
            ident: navaid.ndb_identifier,
            name: navaid.ndb_name,
            frequency: navaid.ndb_frequency,
            location: Coordinates {
                lat: navaid.ndb_latitude,
                long: navaid.ndb_longitude,
            },
            id: navaid.id,
        }
    }
}
