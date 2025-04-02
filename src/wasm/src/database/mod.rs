mod types;
mod utils;

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::Mutex,
};

use rusqlite::{params, params_from_iter, types::ValueRef, Connection, OpenFlags};
use serde_json::{Number, Value};
pub use utils::{Coordinates, NauticalMiles};

pub use types::{
    airport::Airport,
    airspace::{
        map_controlled_airspaces, map_restrictive_airspaces, ControlledAirspace,
        RestrictiveAirspace,
    },
    airway::{map_airways, Airway},
    communication::Communication,
    database_info::DatabaseInfo,
    gate::Gate,
    gls_navaid::GlsNavaid,
    ndb_navaid::NdbNavaid,
    path_point::PathPoint,
    procedure::{
        approach::{map_approaches, Approach},
        arrival::{map_arrivals, Arrival},
        departure::{map_departures, Departure},
    },
    runway::RunwayThreshold,
    sql,
    vhf_navaid::VhfNavaid,
    waypoint::Waypoint,
};

/// The path to the navigation data files folder in the work directory
pub const WORK_NAVIGATION_DATA_FOLDER: &str = "\\work/NavigationData";
/// The path to the "master" cycle info JSON
pub const WORK_CYCLE_JSON_PATH: &str = "\\work/NavigationData/cycle.json";
/// The path to the "master" SQLite DB
pub const WORK_DB_PATH: &str = "\\work/NavigationData/db.s3db";
/// The path to the layout.json in the addon folder
pub const LAYOUT_JSON: &str = ".\\layout.json";
/// The folder name for bundled navigation data
pub const BUNDLED_FOLDER_NAME: &str = "NavigationData";

/// The global exported database state
pub static DATABASE_STATE: Lazy<Mutex<DatabaseState>> =
    Lazy::new(|| Mutex::new(DatabaseState::new().unwrap())); // SAFETY: the only way this function can return an error is if layout.json is corrupt (which is impossible since the package wouldn't even mount), or if copying to the work folder is failing (in which case we have more fundamental problems). So overall, unwrapping here is safe

/// An entry in the layout.json file
#[derive(Deserialize)]
struct LayoutEntry {
    path: String,
}

/// The representation of the layout.json file
#[derive(Deserialize)]
struct LayoutJson {
    content: Vec<LayoutEntry>,
}

/// Find the bundled navigation data distribution
fn get_bundled_db() -> Result<Option<DatabaseDistributionInfo>> {
    // Since we don't know the exact filenames of the bundled navigation data,
    // we need to find them through the layout.json file. In a perfect world,
    // we would just enumerate the bundled directory. However, fd_readdir is unreliable in the sim.
    let mut layout = fs::read_to_string(LAYOUT_JSON)?;
    let parsed = serde_json::from_str::<LayoutJson>(&mut layout)?;

    // Filter out the files in the layout that are not in the bundled folder
    let bundled_files = parsed
        .content
        .iter()
        .filter_map(|e| {
            let path = Path::new(&e.path);

            // Get parent
            let (Some(parent), Some(filename)) = (path.parent(), path.file_name()) else {
                return None;
            };

            // Ensure the file is within our known bundled data path
            if parent != Path::new(BUNDLED_FOLDER_NAME) {
                return None;
            };

            // Finally, return just the basename
            filename.to_str()
        })
        .collect::<Vec<_>>();

    // Try extracting the cycle info and DB files
    let cycle_info = if let Some(file) = bundled_files
        .iter()
        .find(|f| f.to_lowercase().ends_with(".json"))
    {
        file
    } else {
        return Ok(None);
    };

    let db_file = if let Some(file) = bundled_files
        .iter()
        .find(|f| f.to_lowercase().ends_with(".s3db"))
    {
        file
    } else {
        return Ok(None);
    };

    Ok(Some(DatabaseDistributionInfo::new(
        Path::new(&format!(".\\{BUNDLED_FOLDER_NAME}\\{cycle_info}")), // We need to reconstruct the bundled path to include the proper syntax to reference non-work folder files
        Path::new(&format!(".\\{BUNDLED_FOLDER_NAME}\\{db_file}")),
    )?))
}

/// The struct representation of the cycle info JSON
#[derive(Deserialize)]
pub struct CycleInfo {
    pub cycle: String,
    pub revision: String,
    pub format: String,
    #[serde(rename = "validityPeriod")]
    pub validity_period: String,
}

impl CycleInfo {
    /// Attempt to parse from a path
    ///
    /// * `path` - The path to load from
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;

        serde_json::from_reader(&mut file)
            .map_err(|e| anyhow!("error occurred reading cycle.json: {e}"))
    }
}

/// A pair of a cycle info JSON and the corresponding SQLite database.
struct DatabaseDistributionInfo {
    cycle_info: CycleInfo,
    db_path: PathBuf,
    cycle_info_path: PathBuf,
}

impl DatabaseDistributionInfo {
    /// Create a new distribution info set
    ///
    /// * `cycle_info_path` - The path to the cycle info JSON
    /// * `db_path` - The path to the SQLite DB
    pub fn new(cycle_info_path: &Path, db_path: &Path) -> Result<Self> {
        // Ensure paths exist (fs::exists is unreliable, so try getting a handle)
        if File::open(cycle_info_path).is_err() || File::open(db_path).is_err() {
            return Err(anyhow!("invalid distribution path"));
        }

        Ok(Self {
            cycle_info: CycleInfo::from_path(cycle_info_path)?,
            db_path: db_path.to_owned(),
            cycle_info_path: cycle_info_path.to_owned(),
        })
    }
}

/// The overall database state holder
#[derive(Default)]
pub struct DatabaseState {
    database: Option<Connection>,
}

impl DatabaseState {
    /// Create a database state, intended to only be instantiated once (held in the DATABASE_STATE static)
    ///
    /// This searches for the best DB to use by comparing the cycle and revision of both the downloaded (in work folder) and bundled navigation data.
    fn new() -> Result<Self> {
        // Start out with a fresh instance
        let mut instance = Self::default();
        // Get distribution info of both bundled and downloaded DBs, if they exist
        let bundled_distribution = get_bundled_db()?;
        let downloaded_distribution =
            DatabaseDistributionInfo::new(Path::new(WORK_CYCLE_JSON_PATH), Path::new(WORK_DB_PATH))
                .ok();

        // Find the most recent distribution
        let latest = [downloaded_distribution, bundled_distribution]
            .into_iter()
            .filter_map(|d| d)
            .reduce(|a, b| {
                // First, compare by cycle number
                match a
                    .cycle_info
                    .cycle
                    .parse::<u32>()
                    .unwrap_or(0)
                    .cmp(&b.cycle_info.cycle.parse::<u32>().unwrap_or(0))
                {
                    Ordering::Greater => a,
                    Ordering::Less => b,
                    Ordering::Equal => {
                        // If they are somehow equal, compare revisions
                        match a
                            .cycle_info
                            .revision
                            .parse::<u32>()
                            .unwrap_or(0)
                            .cmp(&b.cycle_info.revision.parse::<u32>().unwrap_or(0))
                        {
                            Ordering::Greater | Ordering::Equal => a,
                            Ordering::Less => b,
                        }
                    }
                }
            });

        // If we somehow don't have a cycle in bundled or downloaded, return an empty instance
        let Some(latest) = latest else {
            return Ok(instance);
        };

        // Ensure parent folder exists (ignore the result as it will return an error if it already exists)
        let _ = fs::create_dir_all(WORK_NAVIGATION_DATA_FOLDER);

        // Ensure files get copied over
        if latest.cycle_info_path != PathBuf::from(WORK_CYCLE_JSON_PATH) {
            fs::copy(&latest.cycle_info_path, WORK_CYCLE_JSON_PATH)?;
        }
        if latest.db_path != PathBuf::from(WORK_DB_PATH) {
            fs::copy(&latest.db_path, WORK_DB_PATH)?;
        }

        // The only way this can fail (since we know now that the path is valid) is if the file is corrupt, in which case we should report to sentry
        instance.open_connection()?;

        Ok(instance)
    }

    fn get_database(&self) -> Result<&Connection> {
        self.database.as_ref().ok_or(anyhow!("No database open"))
    }

    pub fn close_connection(&mut self) -> Result<()> {
        if let Some(connection) = self.database.take() {
            connection
                .close()
                .map_err(|_| anyhow!("Unable to close database"))?;
        };

        Ok(())
    }

    pub fn open_connection(&mut self) -> Result<()> {
        // We have to open with flags because the SQLITE_OPEN_CREATE flag with the default open causes the file to
        // be overwritten
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;

        // The WORK_DB_PATH is the "master" SQLite path. We have logic copying over bundled navigation data if needed in the DatabaseState::new function.
        let conn = Connection::open_with_flags(WORK_DB_PATH, flags)?;
        self.database = Some(conn);

        Ok(())
    }

    pub fn get_cycle_info(&self) -> Result<CycleInfo> {
        // The WORK_CYCLE_JSON_PATH is the "master" cycle JSON path.
        return CycleInfo::from_path(Path::new(WORK_CYCLE_JSON_PATH));
    }

    pub fn execute_sql_query(&self, sql: &str, params: &Vec<String>) -> Result<Value> {
        // Execute query
        let conn = self.get_database()?;
        let mut stmt = conn.prepare(sql)?;
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
                    ValueRef::Text(text) => {
                        Some(Value::String(String::from_utf8(text.into()).unwrap()))
                    }
                    ValueRef::Integer(int) => Some(Value::Number(Number::from(int))),
                    ValueRef::Real(real) => Some(Value::Number(Number::from_f64(real).unwrap())),
                    ValueRef::Null => None,
                    ValueRef::Blob(_) => None,
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

    pub fn get_database_info(&self) -> Result<DatabaseInfo> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_hdr_header")?;

        let header_data = utils::fetch_row::<sql::Header>(&mut stmt, params![])?;

        Ok(DatabaseInfo::from(header_data))
    }

    pub fn get_airport(&self, ident: &str) -> Result<Airport> {
        let conn = self.get_database()?;

        let mut stmt =
            conn.prepare("SELECT * FROM tbl_pa_airports WHERE airport_identifier = (?1)")?;

        let airport_data = utils::fetch_row::<sql::Airports>(&mut stmt, params![ident])?;

        Ok(Airport::from(airport_data))
    }

    pub fn get_waypoints(&self, ident: &str) -> Result<Vec<Waypoint>> {
        let conn = self.get_database()?;

        let mut enroute_stmt = conn
            .prepare("SELECT * FROM tbl_ea_enroute_waypoints WHERE waypoint_identifier = (?1)")?;
        let mut terminal_stmt = conn
            .prepare("SELECT * FROM tbl_pc_terminal_waypoints WHERE waypoint_identifier = (?1)")?;

        let enroute_data = utils::fetch_rows::<sql::Waypoints>(&mut enroute_stmt, params![ident])?;
        let terminal_data =
            utils::fetch_rows::<sql::Waypoints>(&mut terminal_stmt, params![ident])?;

        Ok(enroute_data
            .into_iter()
            .chain(terminal_data)
            .map(Waypoint::from)
            .collect())
    }

    pub fn get_vhf_navaids(&self, ident: &str) -> Result<Vec<VhfNavaid>> {
        let conn = self.get_database()?;

        let mut stmt =
            conn.prepare("SELECT * FROM tbl_d_vhfnavaids WHERE navaid_identifier = (?1)")?;

        let navaids_data = utils::fetch_rows::<sql::VhfNavaids>(&mut stmt, params![ident])?;

        Ok(navaids_data.into_iter().map(VhfNavaid::from).collect())
    }

    pub fn get_ndb_navaids(&self, ident: &str) -> Result<Vec<NdbNavaid>> {
        let conn = self.get_database()?;

        let mut enroute_stmt =
            conn.prepare("SELECT * FROM tbl_db_enroute_ndbnavaids WHERE navaid_identifier = (?1)")?;
        let mut terminal_stmt = conn
            .prepare("SELECT * FROM tbl_pn_terminal_ndbnavaids WHERE navaid_identifier = (?1)")?;

        let enroute_data = utils::fetch_rows::<sql::NdbNavaids>(&mut enroute_stmt, params![ident])?;
        let terminal_data =
            utils::fetch_rows::<sql::NdbNavaids>(&mut terminal_stmt, params![ident])?;

        Ok(enroute_data
            .into_iter()
            .chain(terminal_data)
            .map(NdbNavaid::from)
            .collect())
    }

    pub fn get_airways(&self, ident: &str) -> Result<Vec<Airway>> {
        let conn = self.get_database()?;

        let mut stmt =
            conn.prepare("SELECT * FROM tbl_er_enroute_airways WHERE route_identifier = (?1)")?;

        let airways_data = utils::fetch_rows::<sql::EnrouteAirways>(&mut stmt, params![ident])?;

        Ok(map_airways(airways_data))
    }

    pub fn get_airways_at_fix(&self, fix_ident: &str, fix_icao_code: &str) -> Result<Vec<Airway>> {
        let conn = self.get_database()?;

        let mut stmt: rusqlite::Statement<'_> = conn.prepare(
            "SELECT * FROM tbl_er_enroute_airways WHERE route_identifier IN (SELECT route_identifier FROM \
             tbl_er_enroute_airways WHERE waypoint_identifier = (?1) AND icao_code = (?2))",
        )?;
        let all_airways =
            utils::fetch_rows::<sql::EnrouteAirways>(&mut stmt, params![fix_ident, fix_icao_code])?;

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
        &self,
        center: &Coordinates,
        range: &NauticalMiles,
    ) -> Result<Vec<Airport>> {
        let conn = self.get_database()?;

        let where_string = utils::range_query_where(center, *range, "airport_ref");

        let mut stmt =
            conn.prepare(format!("SELECT * FROM tbl_pa_airports WHERE {where_string}").as_str())?;

        let airports_data = utils::fetch_rows::<sql::Airports>(&mut stmt, [])?;

        // Filter into a circle of range
        Ok(airports_data
            .into_iter()
            .map(Airport::from)
            .filter(|airport| airport.location.distance_to(center) <= *range)
            .collect())
    }

    pub fn get_waypoints_in_range(
        &self,
        center: &Coordinates,
        range: &NauticalMiles,
    ) -> Result<Vec<Waypoint>> {
        let conn = self.get_database()?;

        let where_string = utils::range_query_where(center, *range, "waypoint");

        let mut enroute_stmt = conn.prepare(
            format!("SELECT * FROM tbl_ea_enroute_waypoints WHERE {where_string}").as_str(),
        )?;
        let mut terminal_stmt = conn.prepare(
            format!("SELECT * FROM tbl_pc_terminal_waypoints WHERE {where_string}").as_str(),
        )?;

        let enroute_data = utils::fetch_rows::<sql::Waypoints>(&mut enroute_stmt, [])?;
        let terminal_data = utils::fetch_rows::<sql::Waypoints>(&mut terminal_stmt, [])?;

        // Filter into a circle of range
        Ok(enroute_data
            .into_iter()
            .chain(terminal_data)
            .map(Waypoint::from)
            .filter(|waypoint| waypoint.location.distance_to(center) <= *range)
            .collect())
    }

    pub fn get_ndb_navaids_in_range(
        &self,
        center: &Coordinates,
        range: &NauticalMiles,
    ) -> Result<Vec<NdbNavaid>> {
        let conn = self.get_database()?;

        let where_string = utils::range_query_where(center, *range, "navaid");

        let mut enroute_stmt = conn.prepare(
            format!("SELECT * FROM tbl_db_enroute_ndbnavaids WHERE {where_string}").as_str(),
        )?;
        let mut terminal_stmt = conn.prepare(
            format!("SELECT * FROM tbl_pn_terminal_ndbnavaids WHERE {where_string}").as_str(),
        )?;

        let enroute_data = utils::fetch_rows::<sql::NdbNavaids>(&mut enroute_stmt, [])?;
        let terminal_data = utils::fetch_rows::<sql::NdbNavaids>(&mut terminal_stmt, [])?;

        // Filter into a circle of range
        Ok(enroute_data
            .into_iter()
            .chain(terminal_data)
            .map(NdbNavaid::from)
            .filter(|waypoint| waypoint.location.distance_to(center) <= *range)
            .collect())
    }

    pub fn get_vhf_navaids_in_range(
        &self,
        center: &Coordinates,
        range: &NauticalMiles,
    ) -> Result<Vec<VhfNavaid>> {
        let conn = self.get_database()?;

        let where_string = utils::range_query_where(center, *range, "navaid");

        let mut stmt =
            conn.prepare(format!("SELECT * FROM tbl_d_vhfnavaids WHERE {where_string}").as_str())?;

        let navaids_data = utils::fetch_rows::<sql::VhfNavaids>(&mut stmt, [])?;

        // Filter into a circle of range
        Ok(navaids_data
            .into_iter()
            .map(VhfNavaid::from)
            .filter(|navaid| navaid.location.distance_to(center) <= *range)
            .collect())
    }

    pub fn get_airways_in_range(
        &self,
        center: &Coordinates,
        range: &NauticalMiles,
    ) -> Result<Vec<Airway>> {
        let conn = self.get_database()?;

        let where_string = utils::range_query_where(center, *range, "waypoint");

        let mut stmt = conn.prepare(
            format!(
                "SELECT * FROM tbl_er_enroute_airways WHERE route_identifier IN (SELECT route_identifier FROM \
                 tbl_er_enroute_airways WHERE {where_string})"
            )
            .as_str(),
        )?;

        let airways_data = utils::fetch_rows::<sql::EnrouteAirways>(&mut stmt, [])?;

        Ok(map_airways(airways_data)
            .into_iter()
            .filter(|airway| {
                airway
                    .fixes
                    .iter()
                    .any(|fix| fix.location.distance_to(center) <= *range)
            })
            .collect())
    }

    pub fn get_controlled_airspaces_in_range(
        &self,
        center: &Coordinates,
        range: &NauticalMiles,
    ) -> Result<Vec<ControlledAirspace>> {
        let conn = self.get_database()?;

        let where_string = utils::range_query_where(center, *range, "");
        let arc_where_string = utils::range_query_where(center, *range, "arc_origin");

        let range_query = format!(
            "SELECT airspace_center, multiple_code FROM tbl_uc_controlled_airspace WHERE {where_string} OR \
             {arc_where_string}"
        );

        let mut stmt = conn.prepare(
            format!(
                "SELECT * FROM tbl_uc_controlled_airspace WHERE (airspace_center, multiple_code) IN ({range_query})"
            )
            .as_str(),
        )?;

        // No changes since v1, able to use same struct
        let airspaces_data = utils::fetch_rows::<sql::ControlledAirspace>(&mut stmt, [])?;

        Ok(map_controlled_airspaces(airspaces_data))
    }

    pub fn get_restrictive_airspaces_in_range(
        &self,
        center: &Coordinates,
        range: &NauticalMiles,
    ) -> Result<Vec<RestrictiveAirspace>> {
        let conn = self.get_database()?;

        let where_string = utils::range_query_where(center, *range, "");
        let arc_where_string = utils::range_query_where(center, *range, "arc_origin");

        let range_query = format!(
            "SELECT restrictive_airspace_designation, icao_code FROM tbl_ur_restrictive_airspace WHERE {where_string} \
             OR {arc_where_string}"
        );

        let mut stmt = conn.prepare(
            format!(
                "SELECT * FROM tbl_ur_restrictive_airspace WHERE (restrictive_airspace_designation, icao_code) IN \
                 ({range_query})"
            )
            .as_str(),
        )?;

        // No changes since v1, able to use same struct
        let airspaces_data = utils::fetch_rows::<sql::RestrictiveAirspace>(&mut stmt, [])?;

        Ok(map_restrictive_airspaces(airspaces_data))
    }

    pub fn get_communications_in_range(
        &self,
        center: &Coordinates,
        range: &NauticalMiles,
    ) -> Result<Vec<Communication>> {
        let conn = self.get_database()?;

        let where_string = utils::range_query_where(center, *range, "");

        let mut enroute_stmt = conn.prepare(
            format!("SELECT * FROM tbl_ev_enroute_communication WHERE {where_string}").as_str(),
        )?;

        let mut terminal_stmt = conn.prepare(
            format!("SELECT * FROM tbl_pv_airport_communication WHERE {where_string}").as_str(),
        )?;

        let enroute_data = utils::fetch_rows::<sql::EnrouteCommunication>(&mut enroute_stmt, [])?;
        let terminal_data = utils::fetch_rows::<sql::AirportCommunication>(&mut terminal_stmt, [])?;

        Ok(enroute_data
            .into_iter()
            .map(Communication::from)
            .chain(terminal_data.into_iter().map(Communication::from))
            .filter(|waypoint| waypoint.location.distance_to(center) <= *range)
            .collect())
    }

    pub fn get_runways_at_airport(&self, airport_ident: &str) -> Result<Vec<RunwayThreshold>> {
        let conn = self.get_database()?;

        let mut stmt =
            conn.prepare("SELECT * FROM tbl_pg_runways WHERE airport_identifier = (?1)")?;

        let runways_data = utils::fetch_rows::<sql::Runways>(&mut stmt, params![airport_ident])?;

        Ok(runways_data.into_iter().map(Into::into).collect())
    }

    pub fn get_departures_at_airport(&self, airport_ident: &str) -> Result<Vec<Departure>> {
        let conn = self.get_database()?;

        let mut departures_stmt =
            conn.prepare("SELECT * FROM tbl_pd_sids WHERE airport_identifier = (?1)")?;

        let mut runways_stmt =
            conn.prepare("SELECT * FROM tbl_pg_runways WHERE airport_identifier = (?1)")?;

        let departures_data =
            utils::fetch_rows::<sql::Procedures>(&mut departures_stmt, params![airport_ident])?;
        let runways_data =
            utils::fetch_rows::<sql::Runways>(&mut runways_stmt, params![airport_ident])?;

        Ok(map_departures(departures_data, runways_data))
    }

    pub fn get_arrivals_at_airport(&self, airport_ident: &str) -> Result<Vec<Arrival>> {
        let conn = self.get_database()?;

        let mut arrivals_stmt =
            conn.prepare("SELECT * FROM tbl_pe_stars WHERE airport_identifier = (?1)")?;

        let mut runways_stmt =
            conn.prepare("SELECT * FROM tbl_pg_runways WHERE airport_identifier = (?1)")?;

        let arrivals_data =
            utils::fetch_rows::<sql::Procedures>(&mut arrivals_stmt, params![airport_ident])?;
        let runways_data =
            utils::fetch_rows::<sql::Runways>(&mut runways_stmt, params![airport_ident])?;

        Ok(map_arrivals(arrivals_data, runways_data))
    }

    pub fn get_approaches_at_airport(&self, airport_ident: &str) -> Result<Vec<Approach>> {
        let conn = self.get_database()?;

        let mut approachs_stmt =
            conn.prepare("SELECT * FROM tbl_pf_iaps WHERE airport_identifier = (?1)")?;

        let approaches_data =
            utils::fetch_rows::<sql::Procedures>(&mut approachs_stmt, params![airport_ident])?;

        Ok(map_approaches(approaches_data))
    }

    pub fn get_waypoints_at_airport(&self, airport_ident: &str) -> Result<Vec<Waypoint>> {
        let conn = self.get_database()?;

        let mut stmt =
            conn.prepare("SELECT * FROM tbl_pc_terminal_waypoints WHERE region_code = (?1)")?;

        let waypoints_data =
            utils::fetch_rows::<sql::Waypoints>(&mut stmt, params![airport_ident])?;

        Ok(waypoints_data.into_iter().map(Waypoint::from).collect())
    }

    pub fn get_ndb_navaids_at_airport(&self, airport_ident: &str) -> Result<Vec<NdbNavaid>> {
        let conn = self.get_database()?;

        let mut stmt = conn
            .prepare("SELECT * FROM tbl_pn_terminal_ndbnavaids WHERE airport_identifier = (?1)")?;

        let waypoints_data =
            utils::fetch_rows::<sql::NdbNavaids>(&mut stmt, params![airport_ident])?;

        Ok(waypoints_data.into_iter().map(NdbNavaid::from).collect())
    }

    pub fn get_gates_at_airport(&self, airport_ident: &str) -> Result<Vec<Gate>> {
        let conn = self.get_database()?;

        let mut stmt =
            conn.prepare("SELECT * FROM tbl_pb_gates WHERE airport_identifier = (?1)")?;

        // Same as v1, same struct can be used
        let gates_data = utils::fetch_rows::<sql::Gate>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(Gate::from).collect())
    }

    pub fn get_communications_at_airport(&self, airport_ident: &str) -> Result<Vec<Communication>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare(
            "SELECT * FROM tbl_pv_airport_communication WHERE airport_identifier = (?1)",
        )?;

        let gates_data =
            utils::fetch_rows::<sql::AirportCommunication>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(Communication::from).collect())
    }

    pub fn get_gls_navaids_at_airport(&self, airport_ident: &str) -> Result<Vec<GlsNavaid>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pt_gls WHERE airport_identifier = (?1)")?;

        let gates_data = utils::fetch_rows::<sql::Gls>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(GlsNavaid::from).collect())
    }

    pub fn get_path_points_at_airport(&self, airport_ident: &str) -> Result<Vec<PathPoint>> {
        let conn = self.get_database()?;

        let mut stmt =
            conn.prepare("SELECT * FROM tbl_pp_pathpoint WHERE airport_identifier = (?1)")?;

        let gates_data = utils::fetch_rows::<sql::Pathpoints>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(PathPoint::from).collect())
    }
}
