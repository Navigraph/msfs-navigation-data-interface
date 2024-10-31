use std::{error::Error, fs, path::Path};

use rusqlite::{Connection, Result};

use crate::{
    enums::InterfaceFormat,
    output::database_info::DatabaseInfo,
    traits::{DatabaseTrait, InstalledNavigationDataCycleInfo, NoDatabaseOpen, PackageInfo},
};

/// Used for manual connections, only handles setting packages as active
#[derive(Default)]
pub struct DatabaseManual {
    path: String,
}

impl DatabaseTrait for DatabaseManual {
    fn get_database_type(&self) -> InterfaceFormat { InterfaceFormat::Custom }

    fn get_database(&self) -> Result<&Connection, NoDatabaseOpen> { Err(NoDatabaseOpen) }

    fn setup(&self) -> Result<String, Box<dyn Error>> {
        // Nothing goes here preferrably
        Ok(String::from("Setup Complete"))
    }

    fn enable_cycle(&mut self, package: PackageInfo) -> Result<bool, Box<dyn Error>> {
        println!("[NAVIGRAPH]: Set active database to {:?}", &package.path);

        self.path.clone_from(&package.path);

        Ok(true)
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
            .split('/')
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        let header_data = DatabaseInfo::new(cycle.cycle, validity[0].clone(), validity[1].clone(), None, None);

        Ok(header_data)
    }
}

// Empty Connection
