use serde::Serialize;

use crate::database::utils::Coordinates;

#[derive(Serialize, Copy, Clone, PartialEq)]
pub enum FixType {
    #[serde(rename = "A")]
    Airport,
    #[serde(rename = "N")]
    NdbNavaid,
    #[serde(rename = "R")]
    RunwayThreshold,
    #[serde(rename = "G")]
    GlsNavaid,
    #[serde(rename = "I")]
    IlsNavaid,
    #[serde(rename = "V")]
    VhfNavaid,
    #[serde(rename = "W")]
    Waypoint,
    #[serde(rename = "U")]
    Unknown,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone)]
/// Represents a fix which was used as a reference in a procedure or an airway.
///
/// Every `Fix` will have a full data entry as one of these structs somewhere in the database with the same `ident` and
/// `icao_code`:
/// - `Airport`
/// - `NdbNavaid`
/// - `RunwayThreshold`
/// - `GlsNavaid`
/// - `IlsNavaid`
/// - `VhfNavaid`
/// - `Waypoint`
pub struct Fix {
    /// The type of fix
    pub fix_type: Option<FixType>,
    pub fix_code: Option<String>,
    /// The identifier of this fix (not unique), such as `KLAX` or `BI` or `RW17L` or `G07J` or `ISYK` or `YXM` or
    /// `GLENN`
    pub ident: String,
    /// The icao prefix of the region that this fix is in.
    pub icao_code: String,
    /// The geographic location of this fix
    pub location: Coordinates,
    /// The identifier of the airport that this fix is associated with, if any
    pub airport_ident: Option<String>,
}

impl Fix {
    /// Creates a `Fix` by using the latitude and longitude fields, and by parsing the linked id field from a procedure
    /// or airway row.
    pub fn from_row_data(
        lat: f64,
        long: f64,
        ident: String,
        icao_code: String,
        airport_ident: Option<String>,
        ref_table: Option<String>,
        fix_code: Option<String>,
    ) -> Self {
        let fix_type = ref_table.map(|ref_table| match ref_table.as_str() {
            "PA" => FixType::Airport,
            "PN" | "DB" => FixType::NdbNavaid,
            "PG" => FixType::RunwayThreshold,
            "PT" => FixType::GlsNavaid,
            "PI" => FixType::IlsNavaid,
            "D " => FixType::VhfNavaid,
            "EA" | "PC" => FixType::Waypoint,
            _ => FixType::Unknown,
        });

        Self {
            fix_type,
            ident,
            icao_code,
            location: Coordinates { lat, long },
            airport_ident,
            fix_code,
        }
    }
}
