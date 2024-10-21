use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs,
    iter::Cycle,
    path::Path,
};

use rusqlite::{params, params_from_iter, types::ValueRef, Connection, OpenFlags, Result};
use serde::Serialize;
use serde_json::{Number, Value};

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
            approach::{map_approaches, map_approaches_v2, Approach},
            arrival::{map_arrivals, map_arrivals_v2, Arrival},
            departure::{map_departures, map_departures_v2, Departure},
        },
        runway::RunwayThreshold,
        vhf_navaid::VhfNavaid,
        waypoint::Waypoint,
    },
    sql_structs,
    traits::{DatabaseNotCompat, DatabaseTrait, InstalledNavigationDataCycleInfo, NoDatabaseOpen, PackageInfo},
    util, v2,
};

/// Used for manual connections, only handles setting packages as active
pub struct DatabaseManual {
    path: String,
}

impl DatabaseTrait for DatabaseManual {
    fn get_database(&self) -> Result<&Connection, NoDatabaseOpen> {
        Err(NoDatabaseOpen)
    }

    fn setup(&self) -> Result<String, Box<dyn Error>> {
        // Nothing goes here preferrably
        Ok(String::from("Setup Complete"))
    }

    fn enable_cycle(&mut self, package: PackageInfo) -> Result<String, Box<dyn Error>> {
        println!("[NAVIGRAPH]: Set active database to {:?}", &package.path);

        self.path = package.path.clone();

        Ok(String::from(serde_json::to_string(&package).unwrap()))
    }

    fn disable_cycle(&mut self, package: PackageInfo) -> Result<String, Box<dyn Error>> {
        println!("[NAVIGRAPH]: Disabling active database");
        Ok(package.uuid)
    }

    fn get_database_info(&self) -> Result<DatabaseInfo, Box<dyn Error>> {
        let cycle_path = Path::new(&self.path).join("cycle.json");

        let cycle: InstalledNavigationDataCycleInfo =
            serde_json::from_reader(fs::File::open(cycle_path).unwrap()).unwrap();

        let validity = cycle
            .validity_period
            .split("/")
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        let header_data = DatabaseInfo::new(cycle.cycle, validity[0].clone(), validity[1].clone(), None, None);

        Ok(header_data)
    }
}

// Empty Connection
impl Default for DatabaseManual {
    fn default() -> Self {
        Self {
            path: Default::default(),
        }
    }
}
