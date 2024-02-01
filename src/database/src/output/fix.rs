use serde::Serialize;

use crate::math::Coordinates;

#[derive(Serialize, Copy, Clone)]
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
    pub fix_type: FixType,
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
    pub fn from_row_data(lat: f64, long: f64, id: String) -> Self {
        let table = id.split("|").nth(0).unwrap();
        let id = id.split("|").nth(1).unwrap();
        let (airport_identifier, icao_code, ident) =
            if table.starts_with("tbl_terminal") || table == "tbl_localizers_glideslopes" || table == "tbl_gls" {
                (Some(&id[0..4]), &id[4..6], &id[6..])
            } else {
                (None, &id[0..2], &id[2..])
            };

        let fix_type = match table {
            "tbl_airports" => FixType::Airport,
            "tbl_terminal_ndbnavaids" | "tbl_enroute_ndbnavaids" => FixType::NdbNavaid,
            "tbl_runways" => FixType::RunwayThreshold,
            "tbl_gls" => FixType::GlsNavaid,
            "tbl_localizers_glideslopes" => FixType::IlsNavaid,
            "tbl_vhfnavaids" => FixType::VhfNavaid,
            "tbl_enroute_waypoints" | "tbl_terminal_waypoints" => FixType::Waypoint,
            x => panic!("Unexpected table: '{x}'"),
        };

        Self {
            fix_type,
            ident: ident.to_string(),
            icao_code: icao_code.to_string(),
            location: Coordinates { lat, long },
            airport_ident: airport_identifier.map(|s| s.to_string()),
        }
    }
}
