use serde::{Deserialize, Serialize};

use serde_json::Value;

use rusqlite::params;

use std::{cell::RefCell, error::Error, rc::Rc};

use crate::{
    math::Coordinates,
    traits::DatabaseTrait,
    util,
    v2::{self, database::DatabaseV2},
};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default)]
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
    #[default]
    #[serde(rename = "W")]
    Waypoint,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Default)]
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
#[derive(Debug)]
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
    /// The ID of the waypoint, useful for searching in v2
    pub id: String,
}

impl Fix {
    /// Creates a `Fix` by using the latitude and longitude fields, and by parsing the linked id field from a procedure
    /// or airway row.
    pub fn from_row_data(lat: f64, long: f64, id_raw: String) -> Self {
        let table = id_raw.split("|").nth(0).unwrap();
        let id = id_raw.split("|").nth(1).unwrap();
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
            id: id_raw,
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
            id: id_raw,
        })
    }

    /// Used for finding the fixes in v2 (only supports v2)
    pub fn from_ids(database: &DatabaseV2, ids: Vec<String>) -> Result<Vec<Self>, Box<dyn Error>> {
        // let db_type = id_raw.trim().split("=").nth(1).unwrap_or_default();

        let mut pa_list: Vec<String> = Vec::new();
        let mut pn_list: Vec<String> = Vec::new();
        let mut db_list: Vec<String> = Vec::new();
        let mut pg_list: Vec<String> = Vec::new();
        let mut pt_list: Vec<String> = Vec::new();
        let mut pi_list: Vec<String> = Vec::new();
        let mut d_list: Vec<String> = Vec::new();
        let mut ea_list: Vec<String> = Vec::new();
        let mut pc_list: Vec<String> = Vec::new();

        for id in ids {
            let db_type = &id[44..];
            match db_type {
                "PA" => pa_list.push(id),
                "PN" => pn_list.push(id),
                "DB" => db_list.push(id),
                "PG" => pg_list.push(id),
                "PT" => pt_list.push(id),
                "PI" => pi_list.push(id),
                "D " => d_list.push(id),
                "EA" => ea_list.push(id),
                "PC" => pc_list.push(id),
                x => panic!("Unexpected table: '{}'", &id),
            };
        }

        let conn = database.get_database()?;
        let query = format!("SELECT 'A' as fix_type, airport_identifier AS ident, icao_code, airport_identifier AS airport_ident, airport_ref_latitude AS lat, airport_ref_longitude AS long, id FROM tbl_pa_airports WHERE id IN {}
    UNION
SELECT 'N' as fix_type, navaid_identifier AS ident, icao_code, airport_identifier AS airport_ident, navaid_latitude AS lat, navaid_longitude AS long, id FROM tbl_pn_terminal_ndbnavaids WHERE id IN {}
    UNION
SELECT 'N' as fix_type, navaid_identifier AS ident, icao_code, NULL AS airport_ident, navaid_latitude AS lat, navaid_longitude AS long, id FROM tbl_db_enroute_ndbnavaids WHERE id IN {}
    UNION
SELECT 'R' as fix_type, runway_identifier AS ident, icao_code, airport_identifier AS airport_ident, runway_latitude AS lat, runway_longitude AS long, id FROM tbl_pg_runways WHERE id IN {}
    UNION
SELECT 'G' as fix_type, gls_ref_path_identifier AS ident, icao_code, airport_identifier AS airport_ident, station_latitude AS lat, station_longitude AS long, id FROM tbl_pt_gls WHERE id IN {}
    UNION
SELECT 'I' as fix_type, llz_identifier AS ident, icao_code, airport_identifier AS airport_ident, llz_latitude AS lat, llz_longitude AS long, id FROM tbl_pi_localizers_glideslopes WHERE id IN {}
    UNION
SELECT 'V' as fix_type, navaid_identifier AS ident, icao_code, airport_identifier AS airport_ident, navaid_latitude AS lat, navaid_longitude AS long, id FROM tbl_d_vhfnavaids WHERE id IN {}
    UNION
SELECT 'W' as fix_type, waypoint_identifier AS ident, icao_code, region_code AS airport_ident, waypoint_latitude AS lat, waypoint_longitude AS long, id FROM tbl_pc_terminal_waypoints WHERE id IN {}
    UNION
SELECT 'W' as fix_type, waypoint_identifier AS ident, icao_code, NULL AS airport_ident, waypoint_latitude AS lat, waypoint_longitude AS long, id FROM tbl_ea_enroute_waypoints WHERE id IN {}",
format!("(\"{}\")", pa_list.join("\", \"")), format!("(\"{}\")", pn_list.join("\", \"")), format!("(\"{}\")", db_list.join("\", \"")), format!("(\"{}\")", pg_list.join("\", \"")), format!("(\"{}\")", pt_list.join("\", \"")), format!("(\"{}\")", pi_list.join("\", \"")), format!("(\"{}\")", d_list.join("\", \"")), format!("(\"{}\")", pc_list.join("\", \"")), format!("(\"{}\")", ea_list.join("\", \"")),
);
        let mut stmt = conn.prepare(&query)?;
        let data = util::fetch_rows::<v2::sql_structs::FixHelper>(&mut stmt, params![])?;

        let fixes = data
            .into_iter()
            .map(|data| Self {
                location: Coordinates {
                    lat: data.lat,
                    long: data.long,
                },
                fix_type: data.fix_type,
                ident: data.ident,
                icao_code: data.icao_code,
                airport_ident: data.airport_ident,
                id: data.id,
            })
            .collect();

        Ok(fixes)
    }
}
