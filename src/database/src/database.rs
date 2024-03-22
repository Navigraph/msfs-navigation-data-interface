use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use rusqlite::{params, params_from_iter, types::ValueRef, Connection, OpenFlags, Result};
use serde_json::{Number, Value};

use super::output::{airport::Airport, airway::map_airways, procedure::departure::map_departures};
use crate::{
    math::{Coordinates, NauticalMiles},
    output::{
        airspace::{map_controlled_airspaces, map_restrictive_airspaces, ControlledAirspace, RestrictiveAirspace},
        airway::Airway,
        communication::Communication,
        database_info::DatabaseInfo,
        gate::Gate,
        gls_navaid::GlsNavaid,
        ndb_navaid::NdbNavaid,
        path_point::PathPoint,
        procedure::{
            approach::{map_approaches, Approach},
            arrival::{map_arrivals, Arrival},
            departure::Departure,
        },
        runway::RunwayThreshold,
        vhf_navaid::VhfNavaid,
        waypoint::Waypoint,
    },
    sql_structs, util,
};

pub struct Database {
    database: Option<Connection>,
    pub path: Option<String>,
}

#[derive(Debug)]
struct NoDatabaseOpen;

impl Display for NoDatabaseOpen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "No database open")
    }
}

impl Error for NoDatabaseOpen {}

impl Database {
    pub fn new() -> Self {
        Database {
            database: None,
            path: None,
        }
    }

    fn get_database(&self) -> Result<&Connection, NoDatabaseOpen> {
        self.database.as_ref().ok_or(NoDatabaseOpen)
    }

    pub fn set_active_database(&mut self, path: String) -> Result<(), Box<dyn Error>> {
        let path = match util::find_sqlite_file(&path) {
            Ok(new_path) => new_path,
            Err(_) => path,
        };
        println!("[NAVIGRAPH] Setting active database to {}", path);
        self.close_connection();
        if util::is_sqlite_file(&path)? {
            self.open_connection(path.clone())?;
        }
        self.path = Some(path);

        Ok(())
    }

    pub fn open_connection(&mut self, path: String) -> Result<(), Box<dyn Error>> {
        // We have to open with flags because the SQLITE_OPEN_CREATE flag with the default open causes the file to
        // be overwritten
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = Connection::open_with_flags(path, flags)?;
        self.database = Some(conn);

        Ok(())
    }

    pub fn execute_sql_query(&self, sql: String, params: Vec<String>) -> Result<Value, Box<dyn Error>> {
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
                    ValueRef::Text(text) => Some(Value::String(String::from_utf8(text.into()).unwrap())),
                    ValueRef::Integer(int) => Some(Value::Number(Number::from(int))),
                    ValueRef::Real(real) => Some(Value::Number(Number::from_f64(real).unwrap())),
                    ValueRef::Null => None,
                    ValueRef::Blob(_) => panic!("Unexpected value type Blob"),
                };

                if let Some(value) = value {
                    map.insert(name.to_string(), value);
                }
            }
            Ok(Value::Object(map))
        })?;

        let mut data = Vec::new();
        for row in data_iter {
            data.push(row?);
        }

        let json = Value::Array(data);

        Ok(json)
    }

    pub fn get_database_info(&self) -> Result<DatabaseInfo, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_header")?;

        let header_data = Database::fetch_row::<sql_structs::Header>(&mut stmt, params![])?;

        Ok(DatabaseInfo::from(header_data))
    }

    pub fn get_airport(&self, ident: String) -> Result<Airport, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_airports WHERE airport_identifier = (?1)")?;

        let airport_data = Database::fetch_row::<sql_structs::Airports>(&mut stmt, params![ident])?;

        Ok(Airport::from(airport_data))
    }

    pub fn get_waypoints(&self, ident: String) -> Result<Vec<Waypoint>, Box<dyn Error>> {
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

    pub fn get_vhf_navaids(&self, ident: String) -> Result<Vec<VhfNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_vhfnavaids WHERE vor_identifier = (?1)")?;

        let navaids_data = Database::fetch_rows::<sql_structs::VhfNavaids>(&mut stmt, params![ident])?;

        Ok(navaids_data.into_iter().map(VhfNavaid::from).collect())
    }

    pub fn get_ndb_navaids(&self, ident: String) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
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

    pub fn get_airways(&self, ident: String) -> Result<Vec<Airway>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_enroute_airways WHERE route_identifier = (?1)")?;

        let airways_data = Database::fetch_rows::<sql_structs::EnrouteAirways>(&mut stmt, params![ident])?;

        Ok(map_airways(airways_data))
    }

    pub fn get_airways_at_fix(&self, fix_ident: String, fix_icao_code: String) -> Result<Vec<Airway>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt: rusqlite::Statement<'_> = conn.prepare(
            "SELECT * FROM tbl_enroute_airways WHERE route_identifier IN (SELECT route_identifier FROM \
             tbl_enroute_airways WHERE waypoint_identifier = (?1) AND icao_code = (?2))",
        )?;
        let all_airways =
            Database::fetch_rows::<sql_structs::EnrouteAirways>(&mut stmt, params![fix_ident, fix_icao_code])?;

        Ok(map_airways(all_airways)
            .into_iter()
            .filter(|airway| {
                airway
                    .fixes
                    .iter()
                    .any(|fix| fix.ident == fix_ident && fix.icao_code == fix_icao_code)
            })
            .collect())
    }

    pub fn get_airports_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Airport>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = Self::range_query_where(center, range, "airport_ref");

        let mut stmt = conn.prepare(format!("SELECT * FROM tbl_airports WHERE {where_string}").as_str())?;

        let airports_data = Database::fetch_rows::<sql_structs::Airports>(&mut stmt, [])?;

        // Filter into a circle of range
        Ok(airports_data
            .into_iter()
            .map(Airport::from)
            .filter(|airport| airport.location.distance_to(&center) <= range)
            .collect())
    }

    pub fn get_waypoints_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Waypoint>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = Self::range_query_where(center, range, "waypoint");

        let mut enroute_stmt =
            conn.prepare(format!("SELECT * FROM tbl_enroute_waypoints WHERE {where_string}").as_str())?;
        let mut terminal_stmt =
            conn.prepare(format!("SELECT * FROM tbl_terminal_waypoints WHERE {where_string}").as_str())?;

        let enroute_data = Database::fetch_rows::<sql_structs::Waypoints>(&mut enroute_stmt, [])?;
        let terminal_data = Database::fetch_rows::<sql_structs::Waypoints>(&mut terminal_stmt, [])?;

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
    ) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = Self::range_query_where(center, range, "ndb");

        let mut enroute_stmt =
            conn.prepare(format!("SELECT * FROM tbl_enroute_ndbnavaids WHERE {where_string}").as_str())?;
        let mut terminal_stmt =
            conn.prepare(format!("SELECT * FROM tbl_terminal_ndbnavaids WHERE {where_string}").as_str())?;

        let enroute_data = Database::fetch_rows::<sql_structs::NdbNavaids>(&mut enroute_stmt, [])?;
        let terminal_data = Database::fetch_rows::<sql_structs::NdbNavaids>(&mut terminal_stmt, [])?;

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
    ) -> Result<Vec<VhfNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = Self::range_query_where(center, range, "vor");

        let mut stmt = conn.prepare(format!("SELECT * FROM tbl_vhfnavaids WHERE {where_string}").as_str())?;

        let navaids_data = Database::fetch_rows::<sql_structs::VhfNavaids>(&mut stmt, [])?;

        // Filter into a circle of range
        Ok(navaids_data
            .into_iter()
            .map(VhfNavaid::from)
            .filter(|navaid| navaid.location.distance_to(&center) <= range)
            .collect())
    }

    pub fn get_airways_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Airway>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = Self::range_query_where(center, range, "waypoint");

        let mut stmt = conn.prepare(
            format!(
                "SELECT * FROM tbl_enroute_airways WHERE route_identifier IN (SELECT route_identifier FROM \
                 tbl_enroute_airways WHERE {where_string})"
            )
            .as_str(),
        )?;

        let airways_data = Database::fetch_rows::<sql_structs::EnrouteAirways>(&mut stmt, [])?;

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

    pub fn get_controlled_airspaces_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<ControlledAirspace>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = Self::range_query_where(center, range, "");
        let arc_where_string = Self::range_query_where(center, range, "arc_origin");

        let range_query = format!(
            "SELECT airspace_center, multiple_code FROM tbl_controlled_airspace WHERE {where_string} OR \
             {arc_where_string}"
        );

        let mut stmt = conn.prepare(
            format!("SELECT * FROM tbl_controlled_airspace WHERE (airspace_center, multiple_code) IN ({range_query})")
                .as_str(),
        )?;

        let airspaces_data = Database::fetch_rows::<sql_structs::ControlledAirspace>(&mut stmt, [])?;

        Ok(map_controlled_airspaces(airspaces_data))
    }

    pub fn get_restrictive_airspaces_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<RestrictiveAirspace>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = Self::range_query_where(center, range, "");
        let arc_where_string = Self::range_query_where(center, range, "arc_origin");

        let range_query: String = format!(
            "SELECT restrictive_airspace_designation, icao_code FROM tbl_restrictive_airspace WHERE {where_string} OR \
             {arc_where_string}"
        );

        let mut stmt = conn.prepare(
            format!(
                "SELECT * FROM tbl_restrictive_airspace WHERE (restrictive_airspace_designation, icao_code) IN \
                 ({range_query})"
            )
            .as_str(),
        )?;

        let airspaces_data = Database::fetch_rows::<sql_structs::RestrictiveAirspace>(&mut stmt, [])?;

        Ok(map_restrictive_airspaces(airspaces_data))
    }

    pub fn get_communications_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Communication>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = Self::range_query_where(center, range, "");

        let mut enroute_stmt =
            conn.prepare(format!("SELECT * FROM tbl_enroute_communication WHERE {where_string}").as_str())?;

        let mut terminal_stmt =
            conn.prepare(format!("SELECT * FROM tbl_airport_communication WHERE {where_string}").as_str())?;

        let enroute_data = Database::fetch_rows::<sql_structs::EnrouteCommunication>(&mut enroute_stmt, [])?;
        let terminal_data = Database::fetch_rows::<sql_structs::AirportCommunication>(&mut terminal_stmt, [])?;

        Ok(enroute_data
            .into_iter()
            .map(Communication::from)
            .chain(terminal_data.into_iter().map(Communication::from))
            .filter(|waypoint| waypoint.location.distance_to(&center) <= range)
            .collect())
    }

    pub fn get_runways_at_airport(&self, airport_ident: String) -> Result<Vec<RunwayThreshold>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_runways WHERE airport_identifier = (?1)")?;

        let runways_data = Database::fetch_rows::<sql_structs::Runways>(&mut stmt, params![airport_ident])?;

        Ok(runways_data.into_iter().map(Into::into).collect())
    }

    pub fn get_departures_at_airport(&self, airport_ident: String) -> Result<Vec<Departure>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut departures_stmt = conn.prepare("SELECT * FROM tbl_sids WHERE airport_identifier = (?1)")?;

        let mut runways_stmt = conn.prepare("SELECT * FROM tbl_runways WHERE airport_identifier = (?1)")?;

        let departures_data =
            Database::fetch_rows::<sql_structs::Procedures>(&mut departures_stmt, params![airport_ident])?;
        let runways_data = Database::fetch_rows::<sql_structs::Runways>(&mut runways_stmt, params![airport_ident])?;

        Ok(map_departures(departures_data, runways_data))
    }

    pub fn get_arrivals_at_airport(&self, airport_ident: String) -> Result<Vec<Arrival>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut arrivals_stmt = conn.prepare("SELECT * FROM tbl_stars WHERE airport_identifier = (?1)")?;

        let mut runways_stmt = conn.prepare("SELECT * FROM tbl_runways WHERE airport_identifier = (?1)")?;

        let arrivals_data =
            Database::fetch_rows::<sql_structs::Procedures>(&mut arrivals_stmt, params![airport_ident])?;
        let runways_data = Database::fetch_rows::<sql_structs::Runways>(&mut runways_stmt, params![airport_ident])?;

        Ok(map_arrivals(arrivals_data, runways_data))
    }

    pub fn get_approaches_at_airport(&self, airport_ident: String) -> Result<Vec<Approach>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut approachs_stmt = conn.prepare("SELECT * FROM tbl_iaps WHERE airport_identifier = (?1)")?;

        let approaches_data =
            Database::fetch_rows::<sql_structs::Procedures>(&mut approachs_stmt, params![airport_ident])?;

        Ok(map_approaches(approaches_data))
    }

    pub fn get_waypoints_at_airport(&self, airport_ident: String) -> Result<Vec<Waypoint>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_terminal_waypoints WHERE region_code = (?1)")?;

        let waypoints_data = Database::fetch_rows::<sql_structs::Waypoints>(&mut stmt, params![airport_ident])?;

        Ok(waypoints_data.into_iter().map(Waypoint::from).collect())
    }

    pub fn get_ndb_navaids_at_airport(&self, airport_ident: String) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_terminal_ndbnavaids WHERE airport_identifier = (?1)")?;

        let waypoints_data = Database::fetch_rows::<sql_structs::NdbNavaids>(&mut stmt, params![airport_ident])?;

        Ok(waypoints_data.into_iter().map(NdbNavaid::from).collect())
    }

    pub fn get_gates_at_airport(&self, airport_ident: String) -> Result<Vec<Gate>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_gate WHERE airport_identifier = (?1)")?;

        let gates_data = Database::fetch_rows::<sql_structs::Gate>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(Gate::from).collect())
    }

    pub fn get_communications_at_airport(&self, airport_ident: String) -> Result<Vec<Communication>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_airport_communication WHERE airport_identifier = (?1)")?;

        let gates_data = Database::fetch_rows::<sql_structs::AirportCommunication>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(Communication::from).collect())
    }

    pub fn get_gls_navaids_at_airport(&self, airport_ident: String) -> Result<Vec<GlsNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_gls WHERE airport_identifier = (?1)")?;

        let gates_data = Database::fetch_rows::<sql_structs::Gls>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(GlsNavaid::from).collect())
    }

    pub fn get_path_points_at_airport(&self, airport_ident: String) -> Result<Vec<PathPoint>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pathpoints WHERE airport_identifier = (?1)")?;

        let gates_data = Database::fetch_rows::<sql_structs::Pathpoints>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(PathPoint::from).collect())
    }

    fn range_query_where(center: Coordinates, range: NauticalMiles, prefix: &str) -> String {
        let (bottom_left, top_right) = center.distance_bounds(range);

        let prefix = if prefix.is_empty() {
            String::new()
        } else {
            format!("{prefix}_")
        };

        if bottom_left.long > top_right.long {
            format!(
                "{prefix}latitude BETWEEN {} AND {} AND ({prefix}longitude >= {} OR {prefix}longitude <= {})",
                bottom_left.lat, top_right.lat, bottom_left.long, top_right.long
            )
        } else if bottom_left.lat.max(top_right.lat) > 80.0 {
            format!("{prefix}latitude >= {}", bottom_left.lat.min(top_right.lat))
        } else if bottom_left.lat.min(top_right.lat) < -80.0 {
            format!("{prefix}latitude <= {}", bottom_left.lat.max(top_right.lat))
        } else {
            format!(
                "{prefix}latitude BETWEEN {} AND {} AND {prefix}longitude BETWEEN {} AND {}",
                bottom_left.lat, top_right.lat, bottom_left.long, top_right.long
            )
        }
    }

    fn fetch_row<T>(stmt: &mut rusqlite::Statement, params: impl rusqlite::Params) -> Result<T, Box<dyn Error>>
    where
        T: for<'r> serde::Deserialize<'r>,
    {
        let mut rows = stmt.query_and_then(params, |r| serde_rusqlite::from_row::<T>(r))?;
        let row = rows.next().ok_or("No row found")??;
        Ok(row)
    }

    fn fetch_rows<T>(stmt: &mut rusqlite::Statement, params: impl rusqlite::Params) -> Result<Vec<T>, Box<dyn Error>>
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

    pub fn close_connection(&mut self) {
        self.database = None;
    }
}
