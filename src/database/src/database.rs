use std::fmt::{Display, Formatter};

use rusqlite::{params, params_from_iter, types::ValueRef, Connection, OpenFlags, Result};

use super::output::{airport::Airport, airway::map_airways, procedure::departure::map_departures};
use crate::{
    math::{Coordinates, NauticalMiles},
    output::{
        airway::Airway,
        database_info::DatabaseInfo,
        ndb_navaid::NdbNavaid,
        procedure::{
            approach::{map_approaches, Approach},
            arrival::{map_arrivals, Arrival},
            departure::Departure,
        },
        runway::RunwayThreshold,
        vhf_navaid::VhfNavaid,
        waypoint::Waypoint,
    },
    sql_structs,
    util,
};

pub struct Database {
    database: Option<Connection>,
}

#[derive(Debug)]
struct NoDatabaseOpen;

impl Display for NoDatabaseOpen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "No database open") }
}

impl std::error::Error for NoDatabaseOpen {}

impl Database {
    pub fn new() -> Self { Database { database: None } }

    fn get_database(&self) -> Result<&Connection, NoDatabaseOpen> { self.database.as_ref().ok_or(NoDatabaseOpen) }

    pub fn set_active_database(&mut self, mut path: String) -> Result<(), Box<dyn std::error::Error>> {
        // Check if the path is a directory and if it is, search for a sqlite file
        let formatted_path = format!("\\work/{}", path);
        if util::get_path_type(std::path::Path::new(&formatted_path)) == util::PathType::Directory {
            path = util::find_sqlite_file(&formatted_path)?;
        }

        // We have to open with flags because the SQLITE_OPEN_CREATE flag with the default open causes the file to
        // be overwritten
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = Connection::open_with_flags(path, flags)?;
        self.database = Some(conn);

        Ok(())
    }

    pub fn execute_sql_query(
        &self, sql: String, params: Vec<String>,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // Execute query
        let conn = self.get_database()?;
        let mut stmt = conn.prepare(&sql)?;
        let names = stmt
            .column_names()
            .into_iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>();

        // Collect data to be returned
        let data_iter = stmt.query_map(params_from_iter(params), |row| {
            let mut map = serde_json::Map::new();
            for (i, name) in names.iter().enumerate() {
                let value = match row.get_ref(i)? {
                    ValueRef::Text(text) => Some(serde_json::Value::String(String::from_utf8(text.into()).unwrap())),
                    ValueRef::Integer(int) => Some(serde_json::Value::Number(serde_json::Number::from(int))),
                    ValueRef::Real(real) => {
                        Some(serde_json::Value::Number(serde_json::Number::from_f64(real).unwrap()))
                    },
                    ValueRef::Null => None,
                    ValueRef::Blob(_) => panic!("Unexpected value type Blob"),
                };

                if let Some(value) = value {
                    map.insert(name.to_string(), value);
                }
            }
            Ok(serde_json::Value::Object(map))
        })?;

        let mut data = Vec::new();
        for row in data_iter {
            data.push(row?);
        }

        let json = serde_json::Value::Array(data);

        Ok(json)
    }

    pub fn get_database_info(&self) -> Result<DatabaseInfo, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_header")?;

        let header_data = Database::fetch_row::<sql_structs::Header>(&mut stmt, params![])?;

        Ok(DatabaseInfo::from(header_data))
    }

    pub fn get_airport(&self, ident: String) -> Result<Airport, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_airports WHERE airport_identifier = (?1)")?;

        let airport_data = Database::fetch_row::<sql_structs::Airports>(&mut stmt, params![ident])?;

        Ok(Airport::from(airport_data))
    }

    pub fn get_waypoints(&self, ident: String) -> Result<Vec<Waypoint>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut enroute_stmt = conn.prepare("SELECT * FROM tbl_enroute_waypoints WHERE waypoint_identifier = (?1)")?;
        let mut terminal_stmt =
            conn.prepare("SELECT * FROM tbl_terminal_waypoints WHERE waypoint_identifier = (?1)")?;

        let enroute_data = Database::fetch_rows::<sql_structs::Waypoints>(&mut enroute_stmt, params![ident])?;
        let terminal_data = Database::fetch_rows::<sql_structs::Waypoints>(&mut terminal_stmt, params![ident])?;

        Ok(enroute_data
            .into_iter()
            .chain(terminal_data.into_iter())
            .map(Waypoint::from)
            .collect())
    }

    pub fn get_vhf_navaids(&self, ident: String) -> Result<Vec<VhfNavaid>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_vhfnavaids WHERE vor_identifier = (?1)")?;

        let navaids_data = Database::fetch_rows::<sql_structs::VhfNavaids>(&mut stmt, params![ident])?;

        Ok(navaids_data.into_iter().map(VhfNavaid::from).collect())
    }

    pub fn get_ndb_navaids(&self, ident: String) -> Result<Vec<NdbNavaid>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut enroute_stmt = conn.prepare("SELECT * FROM tbl_enroute_ndbnavaids WHERE ndb_identifier = (?1)")?;
        let mut terminal_stmt = conn.prepare("SELECT * FROM tbl_terminal_ndbnavaids WHERE ndb_identifier = (?1)")?;

        let enroute_data = Database::fetch_rows::<sql_structs::NdbNavaids>(&mut enroute_stmt, params![ident])?;
        let terminal_data = Database::fetch_rows::<sql_structs::NdbNavaids>(&mut terminal_stmt, params![ident])?;

        Ok(enroute_data
            .into_iter()
            .chain(terminal_data.into_iter())
            .map(NdbNavaid::from)
            .collect())
    }

    pub fn get_airways(&self, ident: String) -> Result<Vec<Airway>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_enroute_airways WHERE route_identifier = (?1)")?;

        let airways_data = Database::fetch_rows::<sql_structs::EnrouteAirways>(&mut stmt, params![ident])?;

        Ok(map_airways(airways_data))
    }

    pub fn get_airports_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Airport>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let (where_string, params) = Self::range_query_where(center, range, "airport_ref");

        let mut stmt = conn.prepare(format!("SELECT * FROM tbl_airports WHERE {where_string}").as_str())?;

        let airports_data = Database::fetch_rows::<sql_structs::Airports>(&mut stmt, params_from_iter(params))?;

        // Filter into a circle of range
        Ok(airports_data
            .into_iter()
            .map(Airport::from)
            .filter(|airport| airport.location.distance_to(&center) <= range)
            .collect())
    }

    pub fn get_waypoints_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Waypoint>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let (where_string, params) = Self::range_query_where(center, range, "waypoint");

        let mut enroute_stmt =
            conn.prepare(format!("SELECT * FROM tbl_enroute_waypoints WHERE {where_string}").as_str())?;
        let mut terminal_stmt =
            conn.prepare(format!("SELECT * FROM tbl_terminal_waypoints WHERE {where_string}").as_str())?;

        let enroute_data =
            Database::fetch_rows::<sql_structs::Waypoints>(&mut enroute_stmt, params_from_iter(params.clone()))?;
        let terminal_data =
            Database::fetch_rows::<sql_structs::Waypoints>(&mut terminal_stmt, params_from_iter(params))?;

        // Filter into a circle of range
        Ok(enroute_data
            .into_iter()
            .chain(terminal_data.into_iter())
            .map(Waypoint::from)
            .filter(|waypoint| waypoint.location.distance_to(&center) <= range)
            .collect())
    }

    pub fn get_ndb_navaids_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<NdbNavaid>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let (where_string, params) = Self::range_query_where(center, range, "ndb");

        let mut enroute_stmt =
            conn.prepare(format!("SELECT * FROM tbl_enroute_ndbnavaids WHERE {where_string}").as_str())?;
        let mut terminal_stmt =
            conn.prepare(format!("SELECT * FROM tbl_terminal_ndbnavaids WHERE {where_string}").as_str())?;

        let enroute_data =
            Database::fetch_rows::<sql_structs::NdbNavaids>(&mut enroute_stmt, params_from_iter(params.clone()))?;
        let terminal_data =
            Database::fetch_rows::<sql_structs::NdbNavaids>(&mut terminal_stmt, params_from_iter(params))?;

        // Filter into a circle of range
        Ok(enroute_data
            .into_iter()
            .chain(terminal_data.into_iter())
            .map(NdbNavaid::from)
            .filter(|waypoint| waypoint.location.distance_to(&center) <= range)
            .collect())
    }

    pub fn get_vhf_navaids_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<VhfNavaid>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let (where_string, params) = Self::range_query_where(center, range, "vor");

        let mut stmt = conn.prepare(format!("SELECT * FROM tbl_vhfnavaids WHERE {where_string}").as_str())?;

        let navaids_data =
            Database::fetch_rows::<sql_structs::VhfNavaids>(&mut stmt, params_from_iter(params.clone()))?;

        // Filter into a circle of range
        Ok(navaids_data
            .into_iter()
            .map(VhfNavaid::from)
            .filter(|navaid| navaid.location.distance_to(&center) <= range)
            .collect())
    }

    pub fn get_airways_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Airway>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let (where_string, params) = Self::range_query_where(center, range, "waypoint");

        let mut stmt = conn.prepare(
            format!(
                "SELECT * FROM tbl_enroute_airways WHERE route_identifier IN (SELECT route_identifier FROM \
                 tbl_enroute_airways WHERE {where_string})"
            )
            .as_str(),
        )?;

        let airways_data = Database::fetch_rows::<sql_structs::EnrouteAirways>(&mut stmt, params_from_iter(params))?;

        Ok(map_airways(airways_data)
            .into_iter()
            .filter(|airway| {
                airway
                    .fixes
                    .iter()
                    .any(|fix| fix.location.distance_to(&center) <= range)
            })
            .collect())
    }

    pub fn get_runways_at_airport(
        &self, airport_ident: String,
    ) -> Result<Vec<RunwayThreshold>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_runways WHERE airport_identifier = (?1)")?;

        let runways_data = Database::fetch_rows::<sql_structs::Runways>(&mut stmt, params![airport_ident])?;

        Ok(runways_data.into_iter().map(Into::into).collect())
    }

    pub fn get_departures_at_airport(
        &self, airport_ident: String,
    ) -> Result<Vec<Departure>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut departures_stmt = conn.prepare("SELECT * FROM tbl_sids WHERE airport_identifier = (?1)")?;

        let mut runways_stmt = conn.prepare("SELECT * FROM tbl_runways WHERE airport_identifier = (?1)")?;

        let departures_data =
            Database::fetch_rows::<sql_structs::Procedures>(&mut departures_stmt, params![airport_ident])?;
        let runways_data = Database::fetch_rows::<sql_structs::Runways>(&mut runways_stmt, params![airport_ident])?;

        Ok(map_departures(departures_data, runways_data))
    }

    pub fn get_arrivals_at_airport(&self, airport_ident: String) -> Result<Vec<Arrival>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut arrivals_stmt = conn.prepare("SELECT * FROM tbl_stars WHERE airport_identifier = (?1)")?;

        let mut runways_stmt = conn.prepare("SELECT * FROM tbl_runways WHERE airport_identifier = (?1)")?;

        let arrivals_data =
            Database::fetch_rows::<sql_structs::Procedures>(&mut arrivals_stmt, params![airport_ident])?;
        let runways_data = Database::fetch_rows::<sql_structs::Runways>(&mut runways_stmt, params![airport_ident])?;

        Ok(map_arrivals(arrivals_data, runways_data))
    }

    pub fn get_approaches_at_airport(
        &self, airport_ident: String,
    ) -> Result<Vec<Approach>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut approachs_stmt = conn.prepare("SELECT * FROM tbl_iaps WHERE airport_identifier = (?1)")?;

        let approaches_data =
            Database::fetch_rows::<sql_structs::Procedures>(&mut approachs_stmt, params![airport_ident])?;

        Ok(map_approaches(approaches_data))
    }

    pub fn get_waypoints_at_airport(&self, airport_ident: String) -> Result<Vec<Waypoint>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_terminal_waypoints WHERE region_code = (?1)")?;

        let waypoints_data = Database::fetch_rows::<sql_structs::Waypoints>(&mut stmt, params![airport_ident])?;

        Ok(waypoints_data.into_iter().map(Waypoint::from).collect())
    }

    pub fn get_ndb_navaids_at_airport(
        &self, airport_ident: String,
    ) -> Result<Vec<NdbNavaid>, Box<dyn std::error::Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_terminal_ndbnavaids WHERE airport_identifier = (?1)")?;

        let waypoints_data = Database::fetch_rows::<sql_structs::NdbNavaids>(&mut stmt, params![airport_ident])?;

        Ok(waypoints_data.into_iter().map(NdbNavaid::from).collect())
    }

    fn range_query_where(center: Coordinates, range: NauticalMiles, prefix: &str) -> (String, Vec<f64>) {
        let (bottom_left, top_right) = center.distance_bounds(range);

        if bottom_left.long > top_right.long {
            (
                format!(
                    "{prefix}_latitude BETWEEN (?1) AND (?2) AND ({prefix}_longitude >= (?3) OR {prefix}_longitude <= \
                     (?4))"
                ),
                vec![bottom_left.lat, top_right.lat, bottom_left.long, top_right.long],
            )
        } else if bottom_left.lat.max(top_right.lat) > 80.0 {
            (
                format!("{prefix}_latitude >= (?1)"),
                vec![bottom_left.lat.min(top_right.lat)],
            )
        } else if bottom_left.lat.min(top_right.lat) < -80.0 {
            (
                format!("{prefix}_latitude <= (?1)"),
                vec![bottom_left.lat.max(top_right.lat)],
            )
        } else {
            (
                format!("{prefix}_latitude BETWEEN (?1) AND (?2) AND {prefix}_longitude BETWEEN (?3) AND (?4)"),
                vec![bottom_left.lat, top_right.lat, bottom_left.long, top_right.long],
            )
        }
    }

    fn fetch_row<T>(
        stmt: &mut rusqlite::Statement, params: impl rusqlite::Params,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: for<'r> serde::Deserialize<'r>,
    {
        let mut rows = stmt.query_and_then(params, |r| serde_rusqlite::from_row::<T>(r))?;
        let row = rows.next().ok_or("No row found")??;
        Ok(row)
    }

    fn fetch_rows<T>(
        stmt: &mut rusqlite::Statement, params: impl rusqlite::Params,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>>
    where
        T: for<'r> serde::Deserialize<'r>,
    {
        let mut rows = stmt.query_and_then(params, |r| serde_rusqlite::from_row::<T>(r))?;
        let mut data = Vec::new();
        while let Some(row) = rows.next() {
            data.push(row.map_err(|e| e.to_string())?);
        }
        Ok(data)
    }

    pub fn close_connection(&mut self) { self.database = None; }
}
