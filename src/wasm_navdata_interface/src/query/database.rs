use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    dispatcher::{Task, TaskStatus},
    json_structs::params,
    util,
};

use rusqlite::{params, Connection, OpenFlags, Result};

pub struct Database {
    database: RefCell<Option<Connection>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            database: RefCell::new(None),
        }
    }

    pub fn set_active_database(
        self: &Rc<Self>,
        task: Rc<RefCell<Task>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        {
            let params = task
                .borrow()
                .parse_data_as::<params::SetActiveDatabaseParams>()?;

            let mut path = params.path;

            // Check if the path is a directory and if it is, search for a sqlite file
            let formatted_path = format!("\\work/{}", path);
            if util::get_path_type(std::path::Path::new(&formatted_path))
                == util::PathType::Directory
            {
                path = util::find_sqlite_file(&formatted_path)?;
            }

            // We have to open with flags because the SQLITE_OPEN_CREATE flag with the default open causes the file to be overwritten
            let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_URI
                | OpenFlags::SQLITE_OPEN_NO_MUTEX;
            let conn = Connection::open_with_flags(path, flags)?;
            self.database.replace(Some(conn));
        }

        task.borrow_mut().status = TaskStatus::Success(None);

        Ok(())
    }

    pub fn execute_sql_query(
        self: &Rc<Self>,
        task: Rc<RefCell<Task>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = {
            // Parse SQL query
            let params = task
                .borrow()
                .parse_data_as::<params::ExecuteSQLQueryParams>()?;

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
            let data_iter = stmt.query_map(params![], |row| {
                let mut map = serde_json::Map::new();
                for (i, name) in names.iter().enumerate() {
                    let value = row.get_ref(i)?.as_str().unwrap_or("");
                    map.insert(
                        name.to_string(),
                        serde_json::Value::String(value.to_string()),
                    );
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

    pub fn close_connection(&self) {
        self.database.replace(None);
    }
}
