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
use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub struct NoDatabaseOpen;

impl Display for NoDatabaseOpen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "No database open")
    }
}

#[derive(Debug)]
pub struct DatabaseNotCompat;

impl Display for DatabaseNotCompat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function not implemented in database type")
    }
}

impl Error for NoDatabaseOpen {}

impl Error for DatabaseNotCompat {}

#[derive(serde::Serialize)]
pub struct PackageInfo {
    pub path: String,
    pub uuid: String,
    pub cycle: InstalledNavigationDataCycleInfo,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InstalledNavigationDataCycleInfo {
    pub cycle: String,
    pub revision: String,
    pub name: String,
    pub format: String,
    #[serde(rename = "validityPeriod")]
    pub validity_period: String,
}

pub trait InterfaceTrait {
    fn new() -> Self;
}

#[allow(unused_variables)]
pub trait DatabaseTrait {
    fn get_database(&self) -> Result<&Connection, NoDatabaseOpen>;

    // Called after the gauge intializes
    fn setup(&self) -> Result<String, Box<dyn Error>>;

    // Takes a pacakge and switches the 'active' connection to the requested package.
    fn change_cycle(&mut self, package: PackageInfo) -> Result<String, Box<dyn Error>>;

    fn execute_sql_query(&self, sql: String, params: Vec<String>) -> Result<Value, Box<dyn Error>> {
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
    fn get_database_info(&self) -> Result<DatabaseInfo, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_airport(&self, ident: String) -> Result<Airport, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_waypoints(&self, ident: String) -> Result<Vec<Waypoint>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_vhf_navaids(&self, ident: String) -> Result<Vec<VhfNavaid>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_ndb_navaids(&self, ident: String) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_airways(&self, ident: String) -> Result<Vec<Airway>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_airways_at_fix(&self, fix_ident: String, fix_icao_code: String) -> Result<Vec<Airway>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_airports_in_range(&self, center: Coordinates, range: NauticalMiles) -> Result<Vec<Airport>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_waypoints_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Waypoint>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_ndb_navaids_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_vhf_navaids_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<VhfNavaid>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_airways_in_range(&self, center: Coordinates, range: NauticalMiles) -> Result<Vec<Airway>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_controlled_airspaces_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<ControlledAirspace>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_restrictive_airspaces_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<RestrictiveAirspace>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_communications_in_range(
        &self, center: Coordinates, range: NauticalMiles,
    ) -> Result<Vec<Communication>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_runways_at_airport(&self, airport_ident: String) -> Result<Vec<RunwayThreshold>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_departures_at_airport(&self, airport_ident: String) -> Result<Vec<Departure>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_arrivals_at_airport(&self, airport_ident: String) -> Result<Vec<Arrival>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_approaches_at_airport(&self, airport_ident: String) -> Result<Vec<Approach>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_waypoints_at_airport(&self, airport_ident: String) -> Result<Vec<Waypoint>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_ndb_navaids_at_airport(&self, airport_ident: String) -> Result<Vec<NdbNavaid>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_gates_at_airport(&self, airport_ident: String) -> Result<Vec<Gate>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_communications_at_airport(&self, airport_ident: String) -> Result<Vec<Communication>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_gls_navaids_at_airport(&self, airport_ident: String) -> Result<Vec<GlsNavaid>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
    fn get_path_points_at_airport(&self, airport_ident: String) -> Result<Vec<PathPoint>, Box<dyn Error>> {
        Err(Box::new(DatabaseNotCompat))
    }
}
