use serde::Serialize;

use crate::{
    math::{Coordinates, KiloHertz},
    sql_structs,
};

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
pub struct NdbNavaid {
    pub area_code: String,
    pub airport_ident: Option<String>,
    pub icao_code: String,
    pub ident: String,
    pub name: String,
    pub frequency: KiloHertz,
    pub location: Coordinates,
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
        }
    }
}
