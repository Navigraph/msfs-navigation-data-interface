use std::{cell::RefCell, rc::Rc};

use rusqlite::{params_from_iter, types::ValueRef, Connection, OpenFlags, Result};

use super::output::{airport::Airport, airway::map_airways};
use crate::{
    dispatcher::{Task, TaskStatus},
    json_structs::params,
    query::sql_structs,
    util,
};

pub struct Database {
    database: RefCell<Option<Connection>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            database: RefCell::new(None),
        }
    }

    pub fn set_active_database(self: &Rc<Self>, task: Rc<RefCell<Task>>) -> Result<(), Box<dyn std::error::Error>> {
        {
            let params = task.borrow().parse_data_as::<params::SetActiveDatabaseParams>()?;

            let mut path = params.path;

            // Check if the path is a directory and if it is, search for a sqlite file
            let formatted_path = format!("\\work/{}", path);
            if util::get_path_type(std::path::Path::new(&formatted_path)) == util::PathType::Directory {
                path = util::find_sqlite_file(&formatted_path)?;
            }

            // We have to open with flags because the SQLITE_OPEN_CREATE flag with the default open causes the file to
            // be overwritten
            let flags =
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX;
            let conn = Connection::open_with_flags(path, flags)?;
            self.database.replace(Some(conn));
        }

        task.borrow_mut().status = TaskStatus::Success(None);

        Ok(())
    }

    pub fn execute_sql_query(self: &Rc<Self>, task: Rc<RefCell<Task>>) -> Result<(), Box<dyn std::error::Error>> {
        let json = {
            // Parse SQL query
            let params = task.borrow().parse_data_as::<params::ExecuteSQLQueryParams>()?;

            // Execute query
            let borrowed_db = self.database.borrow();
            let conn = borrowed_db.as_ref().ok_or("No database open")?;
            let mut stmt = conn.prepare(&params.sql)?;
            let names = stmt
                .column_names()
                .into_iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>();

            // Collect data to be returned
            let data_iter = stmt.query_map(params_from_iter(params.params), |row| {
                let mut map = serde_json::Map::new();
                for (i, name) in names.iter().enumerate() {
                    let value = match row.get_ref(i)? {
                        ValueRef::Text(text) => {
                            Some(serde_json::Value::String(String::from_utf8(text.into()).unwrap()))
                        },
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

            serde_json::Value::Array(data)
        };

        task.borrow_mut().status = TaskStatus::Success(Some(json));

        Ok(())
    }

    pub fn get_airport(self: &Rc<Self>, task: Rc<RefCell<Task>>) -> Result<(), Box<dyn std::error::Error>> {
        let params = task.borrow().parse_data_as::<params::GetAirportParams>()?;

        let borrowed_db = self.database.borrow();
        let conn = borrowed_db.as_ref().ok_or("No database open")?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_airports WHERE airport_identifier = (?1)")?;

        let airport_data = Database::fetch_row::<sql_structs::Airports>(&mut stmt, &[&params.ident])?;

        // Serialize the airport data
        let json = serde_json::to_value(Airport::from(airport_data))?;

        task.borrow_mut().status = TaskStatus::Success(Some(json));

        Ok(())
    }

    pub fn get_airports_in_range(self: &Rc<Self>, task: Rc<RefCell<Task>>) -> Result<(), Box<dyn std::error::Error>> {
        let params = task.borrow().parse_data_as::<params::GetAirportsInRangeParams>()?;

        let borrowed_db = self.database.borrow();
        let conn = borrowed_db.as_ref().ok_or("No database open")?;

        let mut stmt = conn.prepare(
            "SELECT * FROM tbl_airports WHERE airport_ref_latitude BETWEEN (?1) AND (?2) AND airport_ref_longitude \
             BETWEEN (?3) AND (?4)",
        )?;

        let (bottom_left, top_right) = params.center.distance_bounds(params.range);

        let airports_data = Database::fetch_rows::<sql_structs::Airports>(
            &mut stmt,
            &[&bottom_left.lat, &top_right.lat, &bottom_left.long, &top_right.long],
        )?;

        // Serialize the airport data and filter into a circle of range
        let json = serde_json::to_value(
            airports_data
                .into_iter()
                .map(Airport::from)
                .filter(|airport| airport.location.distance_to(&params.center) <= params.range)
                .collect::<Vec<_>>(),
        )?;

        task.borrow_mut().status = TaskStatus::Success(Some(json));

        Ok(())
    }

    pub fn get_airways(self: &Rc<Self>, task: Rc<RefCell<Task>>) -> Result<(), Box<dyn std::error::Error>> {
        let params = task.borrow().parse_data_as::<params::GetAirwaysParams>()?;

        let borrowed_db = self.database.borrow();
        let conn = borrowed_db.as_ref().ok_or("No database open")?;

        let mut stmt = conn.prepare("SELECT * FROM tbl_enroute_airways WHERE route_identifier = (?1)")?;

        let airways_data = Database::fetch_rows::<sql_structs::EnrouteAirways>(&mut stmt, &[&params.ident])?;

        let json = serde_json::to_value(map_airways(airways_data))?;

        task.borrow_mut().status = TaskStatus::Success(Some(json));

        Ok(())
    }

    pub fn get_airways_in_range(self: &Rc<Self>, task: Rc<RefCell<Task>>) -> Result<(), Box<dyn std::error::Error>> {
        let params = task.borrow().parse_data_as::<params::GetAirwaysInRangeParams>()?;

        let borrowed_db = self.database.borrow();
        let conn = borrowed_db.as_ref().ok_or("No database open")?;

        let mut stmt = conn.prepare(
            "SELECT * FROM tbl_enroute_airways WHERE route_identifier IN (SELECT route_identifier FROM \
             tbl_enroute_airways WHERE waypoint_latitude BETWEEN (?1) AND (?2) AND waypoint_longitude BETWEEN (?3) \
             AND (?4))",
        )?;

        let (bottom_left, top_right) = params.center.distance_bounds(params.range);

        let airways_data = Database::fetch_rows::<sql_structs::EnrouteAirways>(
            &mut stmt,
            &[&bottom_left.lat, &top_right.lat, &bottom_left.long, &top_right.long],
        )?;

        let json = serde_json::to_value(
            map_airways(airways_data)
                .into_iter()
                .filter(|airway| {
                    airway
                        .fixes
                        .iter()
                        .any(|fix| fix.location.distance_to(&params.center) <= params.range)
                })
                .collect::<Vec<_>>(),
        )?;

        task.borrow_mut().status = TaskStatus::Success(Some(json));

        Ok(())
    }

    fn fetch_row<T>(
        stmt: &mut rusqlite::Statement, params: &[&dyn rusqlite::ToSql],
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: for<'r> serde::Deserialize<'r>,
    {
        let mut rows = stmt.query_and_then(params, |r| serde_rusqlite::from_row::<T>(r))?;
        let row = rows.next().ok_or("No row found")??;
        Ok(row)
    }

    fn fetch_rows<T>(
        stmt: &mut rusqlite::Statement, params: &[&dyn rusqlite::ToSql],
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

    pub fn close_connection(&self) { self.database.replace(None); }
}
