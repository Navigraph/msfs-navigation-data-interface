use std::{error::Error, path::Path};

use rusqlite::{params, Connection, OpenFlags, Result};

use crate::{
    math::{Coordinates, NauticalMiles},
    output::{
        airport::Airport,
        airspace::{map_controlled_airspaces, map_restrictive_airspaces, ControlledAirspace, RestrictiveAirspace},
        airway::{map_airways_v2, Airway},
        communication::Communication,
        database_info::DatabaseInfo,
        gate::Gate,
        gls_navaid::GlsNavaid,
        ndb_navaid::NdbNavaid,
        path_point::PathPoint,
        procedure::{
            approach::{map_approaches_v2, Approach},
            arrival::{map_arrivals_v2, Arrival},
            departure::{map_departures_v2, Departure},
        },
        runway::RunwayThreshold,
        vhf_navaid::VhfNavaid,
        waypoint::Waypoint,
    },
    sql_structs,
    traits::{DatabaseTrait, NoDatabaseOpen, PackageInfo},
    util, v2,
};

#[derive(Default)]
pub struct DatabaseV2 {
    connection: Option<Connection>,
    pub path: Option<String>,
}

impl DatabaseTrait for DatabaseV2 {
    fn get_database(&self) -> Result<&Connection, NoDatabaseOpen> {
        self.connection.as_ref().ok_or(NoDatabaseOpen)
    }

    fn setup(&self) -> Result<String, Box<dyn Error>> {
        // Nothing goes here preferrably
        Ok(String::from("Setup Complete"))
    }

    fn enable_cycle(&mut self, package: PackageInfo) -> bool {
        let db_path = Path::new("")
            .join(package.path.clone())
            .join(format!("ng_jeppesen_fwdfd_{}.s3db", package.cycle.cycle));

        println!("[NAVIGRAPH]: Setting active database to {:?}", db_path);

        if self.connection.is_some() {
            self.disable_cycle(package.clone()).unwrap();
        }

        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = Connection::open_with_flags(db_path.clone(), flags).unwrap();

        self.connection = Some(conn);
        self.path = Some(String::from(db_path.to_str().unwrap()));

        println!("[NAVIGRAPH]: Set active database to {:?}", db_path);

        true
    }

    fn disable_cycle(&mut self, package: PackageInfo) -> Result<String, Box<dyn Error>> {
        println!("[NAVIGRAPH]: Disabling active database");
        self.connection = None;
        Ok(package.uuid)
    }

    fn get_database_info(&self) -> Result<DatabaseInfo, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_hdr_header")?;

        let header_data = util::fetch_row::<v2::sql_structs::Header>(&mut stmt, params![])?;

        Ok(DatabaseInfo::from(header_data))
    }

    // v2 Compat
    fn get_airport(&self, ident: String) -> Result<Airport, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pa_airports WHERE airport_identifier = (?1)")?;

        let airport_data = util::fetch_row::<v2::sql_structs::Airports>(&mut stmt, params![ident])?;

        Ok(Airport::from(airport_data))
    }

    // v2 Compat
    fn get_waypoints(&self, ident: String) -> Result<Vec<Waypoint>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut enroute_stmt =
            conn.prepare("SELECT * FROM tbl_ea_enroute_waypoints WHERE waypoint_identifier = (?1)")?;
        let mut terminal_stmt =
            conn.prepare("SELECT * FROM tbl_pc_terminal_waypoints WHERE waypoint_identifier = (?1)")?;

        let enroute_data = util::fetch_rows::<v2::sql_structs::Waypoints>(&mut enroute_stmt, params![ident])?;
        let terminal_data = util::fetch_rows::<v2::sql_structs::Waypoints>(&mut terminal_stmt, params![ident])?;

        Ok(enroute_data
            .into_iter()
            .chain(terminal_data)
            .map(Waypoint::from)
            .collect())
    }

    // v2 Compat
    fn get_vhf_navaids(&self, ident: String) -> Result<Vec<VhfNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_d_vhfnavaids WHERE vor_identifier = (?1)")?;

        let navaids_data = util::fetch_rows::<v2::sql_structs::VhfNavaids>(&mut stmt, params![ident])?;

        Ok(navaids_data.into_iter().map(VhfNavaid::from).collect())
    }

    // v2 Compat
    fn get_ndb_navaids(&self, ident: String) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut enroute_stmt = conn.prepare("SELECT * FROM tbl_db_enroute_ndbnavaids WHERE ndb_identifier = (?1)")?;
        let mut terminal_stmt = conn.prepare("SELECT * FROM tbl_pn_terminal_ndbnavaids WHERE ndb_identifier = (?1)")?;

        let enroute_data = util::fetch_rows::<v2::sql_structs::NdbNavaids>(&mut enroute_stmt, params![ident])?;
        let terminal_data = util::fetch_rows::<v2::sql_structs::NdbNavaids>(&mut terminal_stmt, params![ident])?;

        Ok(enroute_data
            .into_iter()
            .chain(terminal_data)
            .map(NdbNavaid::from)
            .collect())
    }

    // v2 Compat
    fn get_airways(&self, ident: String) -> Result<Vec<Airway>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_er_enroute_airways WHERE route_identifier = (?1)")?;

        let airways_data = util::fetch_rows::<v2::sql_structs::EnrouteAirways>(&mut stmt, params![ident])?;

        Ok(map_airways_v2(airways_data))
    }

    // v2 Compat, might have performance issue
    fn get_airways_at_fix(&self, fix_ident: String, fix_icao_code: String) -> Result<Vec<Airway>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt: rusqlite::Statement<'_> = conn.prepare(
            "SELECT * FROM tbl_er_enroute_airways WHERE route_identifier IN (SELECT route_identifier FROM \
             tbl_er_enroute_airways WHERE waypoint_identifier = (?1) AND icao_code = (?2))",
        )?;
        let all_airways =
            util::fetch_rows::<v2::sql_structs::EnrouteAirways>(&mut stmt, params![fix_ident, fix_icao_code])?;

        Ok(map_airways_v2(all_airways)
            .into_iter()
            .filter(|airway| {
                airway
                    .fixes
                    .iter()
                    .any(|fix| fix.ident == fix_ident && fix.icao_code == fix_icao_code)
            })
            .collect())
    }

    // should work, untested
    fn get_airports_in_range(&self, center: Coordinates, range: NauticalMiles) -> Result<Vec<Airport>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = util::range_query_where(center, range, "airport_ref");

        let mut stmt = conn.prepare(format!("SELECT * FROM tbl_pa_airports WHERE {where_string}").as_str())?;

        let airports_data = util::fetch_rows::<v2::sql_structs::Airports>(&mut stmt, [])?;

        // Filter into a circle of range
        Ok(airports_data
            .into_iter()
            .map(Airport::from)
            .filter(|airport| airport.location.distance_to(&center) <= range)
            .collect())
    }

    // should work, untested
    fn get_waypoints_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Waypoint>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = util::range_query_where(center, range, "waypoint");

        let mut enroute_stmt =
            conn.prepare(format!("SELECT * FROM tbl_ea_enroute_waypoints WHERE {where_string}").as_str())?;
        let mut terminal_stmt =
            conn.prepare(format!("SELECT * FROM tbl_pc_terminal_waypoints WHERE {where_string}").as_str())?;

        let enroute_data = util::fetch_rows::<v2::sql_structs::Waypoints>(&mut enroute_stmt, [])?;
        let terminal_data = util::fetch_rows::<v2::sql_structs::Waypoints>(&mut terminal_stmt, [])?;

        // Filter into a circle of range
        Ok(enroute_data
            .into_iter()
            .chain(terminal_data)
            .map(Waypoint::from)
            .filter(|waypoint| waypoint.location.distance_to(&center) <= range)
            .collect())
    }

    // should work, untested
    fn get_ndb_navaids_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = util::range_query_where(center, range, "ndb");

        let mut enroute_stmt =
            conn.prepare(format!("SELECT * FROM tbl_db_enroute_ndbnavaids WHERE {where_string}").as_str())?;
        let mut terminal_stmt =
            conn.prepare(format!("SELECT * FROM tbl_pn_terminal_ndbnavaids WHERE {where_string}").as_str())?;

        let enroute_data = util::fetch_rows::<v2::sql_structs::NdbNavaids>(&mut enroute_stmt, [])?;
        let terminal_data = util::fetch_rows::<v2::sql_structs::NdbNavaids>(&mut terminal_stmt, [])?;

        // Filter into a circle of range
        Ok(enroute_data
            .into_iter()
            .chain(terminal_data)
            .map(NdbNavaid::from)
            .filter(|waypoint| waypoint.location.distance_to(&center) <= range)
            .collect())
    }

    // should work, untested
    fn get_vhf_navaids_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<VhfNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = util::range_query_where(center, range, "vor");

        let mut stmt = conn.prepare(format!("SELECT * FROM tbl_d_vhfnavaids WHERE {where_string}").as_str())?;

        let navaids_data = util::fetch_rows::<v2::sql_structs::VhfNavaids>(&mut stmt, [])?;

        // Filter into a circle of range
        Ok(navaids_data
            .into_iter()
            .map(VhfNavaid::from)
            .filter(|navaid| navaid.location.distance_to(&center) <= range)
            .collect())
    }

    // should work, untested
    fn get_airways_in_range(&self, center: Coordinates, range: NauticalMiles) -> Result<Vec<Airway>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = util::range_query_where(center, range, "waypoint");

        let mut stmt = conn.prepare(
            format!(
                "SELECT * FROM tbl_er_enroute_airways WHERE route_identifier IN (SELECT route_identifier FROM \
                 tbl_er_enroute_airways WHERE {where_string})"
            )
            .as_str(),
        )?;

        let airways_data = util::fetch_rows::<v2::sql_structs::EnrouteAirways>(&mut stmt, [])?;

        Ok(map_airways_v2(airways_data)
            .into_iter()
            .filter(|airway| {
                airway
                    .fixes
                    .iter()
                    .any(|fix| fix.location.distance_to(&center) <= range)
            })
            .collect())
    }

    // should work, untested
    fn get_controlled_airspaces_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<ControlledAirspace>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = util::range_query_where(center, range, "");
        let arc_where_string = util::range_query_where(center, range, "arc_origin");

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
        let airspaces_data = util::fetch_rows::<sql_structs::ControlledAirspace>(&mut stmt, [])?;

        Ok(map_controlled_airspaces(airspaces_data))
    }

    // should work, untested
    fn get_restrictive_airspaces_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<RestrictiveAirspace>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = util::range_query_where(center, range, "");
        let arc_where_string = util::range_query_where(center, range, "arc_origin");

        let range_query: String = format!(
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
        let airspaces_data = util::fetch_rows::<sql_structs::RestrictiveAirspace>(&mut stmt, [])?;

        Ok(map_restrictive_airspaces(airspaces_data))
    }

    // should work, untested
    fn get_communications_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Communication>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let where_string = util::range_query_where(center, range, "");

        let mut enroute_stmt =
            conn.prepare(format!("SELECT * FROM tbl_ev_enroute_communication WHERE {where_string}").as_str())?;

        let mut terminal_stmt =
            conn.prepare(format!("SELECT * FROM tbl_pv_airport_communication WHERE {where_string}").as_str())?;

        let enroute_data = util::fetch_rows::<v2::sql_structs::EnrouteCommunication>(&mut enroute_stmt, [])?;
        let terminal_data = util::fetch_rows::<v2::sql_structs::AirportCommunication>(&mut terminal_stmt, [])?;

        Ok(enroute_data
            .into_iter()
            .map(Communication::from)
            .chain(terminal_data.into_iter().map(Communication::from))
            .filter(|waypoint| waypoint.location.distance_to(&center) <= range)
            .collect())
    }

    // should work, untested
    fn get_runways_at_airport(&self, airport_ident: String) -> Result<Vec<RunwayThreshold>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pg_runways WHERE airport_identifier = (?1)")?;

        let runways_data = util::fetch_rows::<v2::sql_structs::Runways>(&mut stmt, params![airport_ident])?;

        Ok(runways_data.into_iter().map(Into::into).collect())
    }

    // v2 Compat
    fn get_departures_at_airport(&self, airport_ident: String) -> Result<Vec<Departure>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut departures_stmt = conn.prepare("SELECT * FROM tbl_pd_sids WHERE airport_identifier = (?1)")?;

        let mut runways_stmt = conn.prepare("SELECT * FROM tbl_pg_runways WHERE airport_identifier = (?1)")?;

        let departures_data =
            util::fetch_rows::<v2::sql_structs::Procedures>(&mut departures_stmt, params![airport_ident])?;
        let runways_data = util::fetch_rows::<v2::sql_structs::Runways>(&mut runways_stmt, params![airport_ident])?;

        Ok(map_departures_v2(departures_data, runways_data))
    }

    // should work, untested
    fn get_arrivals_at_airport(&self, airport_ident: String) -> Result<Vec<Arrival>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut arrivals_stmt = conn.prepare("SELECT * FROM tbl_pe_stars WHERE airport_identifier = (?1)")?;

        let mut runways_stmt = conn.prepare("SELECT * FROM tbl_pg_runways WHERE airport_identifier = (?1)")?;

        let arrivals_data =
            util::fetch_rows::<v2::sql_structs::Procedures>(&mut arrivals_stmt, params![airport_ident])?;
        let runways_data = util::fetch_rows::<v2::sql_structs::Runways>(&mut runways_stmt, params![airport_ident])?;

        Ok(map_arrivals_v2(arrivals_data, runways_data))
    }

    // should work, untested
    fn get_approaches_at_airport(&self, airport_ident: String) -> Result<Vec<Approach>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut approachs_stmt = conn.prepare("SELECT * FROM tbl_pf_iaps WHERE airport_identifier = (?1)")?;

        let approaches_data =
            util::fetch_rows::<v2::sql_structs::Procedures>(&mut approachs_stmt, params![airport_ident])?;

        Ok(map_approaches_v2(approaches_data))
    }

    // should work, untested
    fn get_waypoints_at_airport(&self, airport_ident: String) -> Result<Vec<Waypoint>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pc_terminal_waypoints WHERE region_code = (?1)")?;

        let waypoints_data = util::fetch_rows::<v2::sql_structs::Waypoints>(&mut stmt, params![airport_ident])?;

        Ok(waypoints_data.into_iter().map(Waypoint::from).collect())
    }

    // should work, untested
    fn get_ndb_navaids_at_airport(&self, airport_ident: String) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pn_terminal_ndbnavaids WHERE airport_identifier = (?1)")?;

        let waypoints_data = util::fetch_rows::<v2::sql_structs::NdbNavaids>(&mut stmt, params![airport_ident])?;

        Ok(waypoints_data.into_iter().map(NdbNavaid::from).collect())
    }

    // should work, untested
    fn get_gates_at_airport(&self, airport_ident: String) -> Result<Vec<Gate>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pb_gate WHERE airport_identifier = (?1)")?;

        // Same as v1, same struct can be used
        let gates_data = util::fetch_rows::<sql_structs::Gate>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(Gate::from).collect())
    }

    // should work, untested
    fn get_communications_at_airport(&self, airport_ident: String) -> Result<Vec<Communication>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pv_airport_communication WHERE airport_identifier = (?1)")?;

        let gates_data = util::fetch_rows::<v2::sql_structs::AirportCommunication>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(Communication::from).collect())
    }

    // should work, untested
    fn get_gls_navaids_at_airport(&self, airport_ident: String) -> Result<Vec<GlsNavaid>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pt_gls WHERE airport_identifier = (?1)")?;

        let gates_data = util::fetch_rows::<v2::sql_structs::Gls>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(GlsNavaid::from).collect())
    }

    fn get_path_points_at_airport(&self, airport_ident: String) -> Result<Vec<PathPoint>, Box<dyn Error>> {
        let conn = self.get_database()?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_pp_pathpoints WHERE airport_identifier = (?1)")?;

        let gates_data = util::fetch_rows::<v2::sql_structs::Pathpoints>(&mut stmt, params![airport_ident])?;

        Ok(gates_data.into_iter().map(PathPoint::from).collect())
    }
}

// Empty Connection
