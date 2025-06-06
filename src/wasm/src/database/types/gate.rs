use serde::Serialize;

use crate::database::utils::Coordinates;

use super::sql;

#[derive(Serialize)]
/// Represents a gate at an airport
pub struct Gate {
    /// The Geographic region where this gate is
    pub area_code: String,
    /// The icao prefix of the airport which this gate is at
    pub icao_code: String,
    /// The identifier of this gate
    pub ident: String,
    /// The location of this gate
    pub location: Coordinates,
    /// The formal name of this gate (usually the same as `ident`)
    pub name: String,
}

impl From<sql::Gate> for Gate {
    fn from(row: sql::Gate) -> Self {
        Self {
            area_code: row.area_code,
            icao_code: row.icao_code,
            ident: row.gate_identifier,
            location: Coordinates {
                lat: row.gate_latitude,
                long: row.gate_longitude,
            },
            name: row.name.unwrap_or(String::from("N/A")),
        }
    }
}
