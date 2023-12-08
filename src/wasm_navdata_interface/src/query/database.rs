use std::cell::RefCell;
use std::{io::Read, rc::Rc};

use crate::{
    dispatcher::{Request, RequestStatus},
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

    fn find_sqlite_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
        // From 1.3.1 of https://www.sqlite.org/fileformat.html
        let sqlite_header = [
            0x53, 0x51, 0x4c, 0x69, 0x74, 0x65, 0x20, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x20,
            0x33, 0x00,
        ];
        // we are going to search this directory for a database
        for entry in std::fs::read_dir("\\work/avionics_v2")? {
            let entry = entry?;
            let path = entry.path();
            if util::get_path_type(&path) == util::PathType::File {
                let path = path.to_str().ok_or("Invalid path")?;
                // get first 16 bytes of file
                let mut file = std::fs::File::open(path)?;
                let mut buf = [0; 16];
                file.read_exact(buf.as_mut())?;
                // compare bytes to sqlite header
                if buf == sqlite_header {
                    // we found a database
                    return Ok(path.to_string());
                }
            }
        }
        Err("No database found".into())
    }

    pub fn set_active_database(self: &Rc<Self>, request: Rc<RefCell<Request>>) {
        // In its own scope so that we can drop the borrow of request
        {
            let json = request.borrow().args.clone();
            let path = json["path"].as_str();
            if path.is_none() {
                request.borrow_mut().status =
                    RequestStatus::Failure("No path provided".to_string());
                return;
            }
            let mut path = path.unwrap().to_owned();

            // Check if the path is a directory and if it is, search for a sqlite file
            let formatted_path = format!("\\work/{}", path);
            if util::get_path_type(std::path::Path::new(&formatted_path))
                == util::PathType::Directory
            {
                match Database::find_sqlite_file(&formatted_path) {
                    Ok(returned_path) => {
                        path = returned_path;
                    }
                    Err(e) => {
                        println!("Failed to find sqlite file: {}", e);
                    }
                }
            }

            let res = self.try_open(&path);
            if res.is_err() {
                request.borrow_mut().status =
                    RequestStatus::Failure(res.err().unwrap().to_string());
                return;
            }
            println!("Opened database at {}", path);
        }
        request.borrow_mut().status = RequestStatus::Success(None);
    }

    pub fn execute_sql_query(self: &Rc<Self>, request: Rc<RefCell<Request>>) {
        let mut sql = String::new();
        {
            let args_json = request.borrow().args.clone();
            let parsed_sql = args_json["sql"].as_str();
            if parsed_sql.is_none() {
                request.borrow_mut().status = RequestStatus::Failure("No SQL provided".to_string());
                return;
            }
            sql = parsed_sql.unwrap().to_string();
        }

        let res = self.try_execute_sql_query(&sql);

        match res {
            Ok(json) => {
                request.borrow_mut().status = RequestStatus::Success(Some(json));
            }
            Err(e) => {
                request.borrow_mut().status = RequestStatus::Failure(e.to_string());
            }
        }
    }

    fn try_execute_sql_query(
        self: &Rc<Self>,
        sql: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let borrowed_db = self.database.borrow();
        let conn = borrowed_db.as_ref().ok_or("No database open")?;
        let mut stmt = conn.prepare(sql)?;
        let names = stmt
            .column_names()
            .into_iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>();
        // let's collect the names and values into a vector of maps
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

        // collect data into a vector
        let mut data = Vec::new();
        for row in data_iter {
            data.push(row?);
        }

        let json = serde_json::json!({
            "data": data,
        });

        Ok(json)
    }

    fn try_open(self: &Rc<Self>, path: &str) -> Result<()> {
        // We have to open with flags because the SQLITE_OPEN_CREATE flag with the default open causes the file to be overwritten
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let conn = Connection::open_with_flags(path, flags)?;
        self.database.replace(Some(conn));
        Ok(())
    }
}
