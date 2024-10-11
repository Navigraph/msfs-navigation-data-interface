use serde::Serialize;

use rusqlite::params;

use std::{cell::RefCell, error::Error};

use crate::{
    math::Coordinates,
    traits::DatabaseTrait,
    util,
    v2::{self, database::DatabaseV2},
};

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
    /// The geographic location of this fix (this does not exist on v2)
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

    /// Used for finding the fix in v2 (only supports v2)
    pub fn from_id(database: &DatabaseV2, id_raw: String) -> Result<Self, Box<dyn Error>> {
        let db_type = id_raw.trim().split("=").nth(1).unwrap_or_default();

        // SQL String Builder, used to generate the queries to fetch the data.
        // Deemed better to use this than write out the whole function for each one.
        let (fix_type, ident_field, airport_ident_field, lat_field, long_field, tbl) = match db_type {
            "PA" => (
                FixType::Airport,
                "airport_identifier",
                "airport_identifier",
                "airport_ref_latitude",
                "airport_ref_longitude",
                "tbl_pa_airports",
            ),
            "PN" | "DB" => (
                FixType::NdbNavaid,
                "navaid_identifier",
                match db_type {
                    "PN" => "airport_identifier",
                    _ => "NULL",
                },
                "navaid_latitude",
                "navaid_longitude",
                match db_type {
                    "PN" => "tbl_pn_terminal_ndbnavaids",
                    _ => "tbl_db_enroute_ndbnavaids",
                },
            ),
            "PG" => (
                FixType::RunwayThreshold,
                "runway_identifier",
                "airport_identifier",
                "runway_latitude",
                "runway_longitude",
                "tbl_pg_runways",
            ),
            "PT" => (
                FixType::GlsNavaid,
                "gls_ref_path_identifier",
                "airport_identifier",
                "station_latitude",
                "station_longitude",
                "tbl_pt_gls",
            ),
            "PI" => (
                FixType::IlsNavaid,
                "llz_identifier",
                "airport_identifier",
                "llz_latitude",
                "llz_longitude",
                "tbl_pi_localizers_glideslopes",
            ),
            "D" => (
                FixType::VhfNavaid,
                "navaid_identifier",
                "airport_identifier",
                "navaid_latitude",
                "navaid_longitude",
                "tbl_d_vhfnavaids",
            ),
            "EA" | "PC" => (
                FixType::Waypoint,
                "waypoint_identifier",
                match db_type {
                    "PC" => "region_code",
                    _ => "NULL",
                },
                "waypoint_latitude",
                "waypoint_longitude",
                match db_type {
                    "EA" => "tbl_ea_enroute_waypoints",
                    _ => "tbl_pc_terminal_waypoints",
                },
            ),
            x => panic!("Unexpected table: '{}'", &id_raw),
        };

        let conn = database.get_database()?;
        let query = format!(
            "SELECT {ident_field} AS ident, icao_code, {airport_ident_field} as airport_ident, {lat_field} AS lat, {long_field} AS long FROM {tbl} WHERE id = (?1)"
        );
        let mut stmt = conn.prepare(&query)?;
        let data = util::fetch_row::<v2::sql_structs::FixHelper>(&mut stmt, params![id_raw])?;

        Ok(Self {
            location: Coordinates {
                lat: data.lat,
                long: data.long,
            },
            fix_type,
            ident: data.ident,
            icao_code: data.icao_code,
            airport_ident: data.airport_ident,
        })
    }
}
