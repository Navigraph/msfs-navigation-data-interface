use crate::math::{Coordinates, NauticalMiles};

use std::{error::Error, fs, io::Read, path::Path};

// From 1.3.1 of https://www.sqlite.org/fileformat.html
const SQLITE_HEADER: [u8; 16] = [
    0x53, 0x51, 0x4c, 0x69, 0x74, 0x65, 0x20, 0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x20, 0x33, 0x00,
];

#[derive(PartialEq, Eq)]
pub enum PathType {
    File,
    Directory,
    DoesNotExist,
}

/// We aren't able to get file metadata in the sim so we can't use some of the standard library file system functions
/// (like is_dir, exists, and some others)
pub fn get_path_type(path: &Path) -> PathType {
    match fs::read_dir(path) {
        Ok(mut dir_res) => {
            let next = dir_res.next();

            if let Some(result) = next {
                if result.is_ok() {
                    return PathType::Directory;
                }
            }
        },
        Err(_) => {},
    };

    let file_res = fs::File::open(path);
    if file_res.is_ok() {
        return PathType::File;
    }

    PathType::DoesNotExist
}

pub fn find_sqlite_file(path: &str) -> Result<String, Box<dyn Error>> {
    if get_path_type(&Path::new(path)) != PathType::Directory {
        return Err("Path is not a directory".into());
    }

    // We are going to search this directory for a database
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if get_path_type(&path) == PathType::File {
            let path = path.to_str().ok_or("Invalid path")?;

            if is_sqlite_file(path)? {
                return Ok(path.to_string());
            }
        }
    }
    Err("No SQL database found. Make sure the database specified is a SQL database".into())
}

pub fn is_sqlite_file(path: &str) -> Result<bool, Box<dyn Error>> {
    if get_path_type(&Path::new(path)) != PathType::File {
        return Ok(false);
    }

    let mut file = fs::File::open(path)?;
    let mut buf = [0; 16];
    file.read_exact(&mut buf)?;
    Ok(buf == SQLITE_HEADER)
}

pub fn range_query_where(center: Coordinates, range: NauticalMiles, prefix: &str) -> String {
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
pub fn fetch_row<T>(stmt: &mut rusqlite::Statement, params: impl rusqlite::Params) -> Result<T, Box<dyn Error>>
where
    T: for<'r> serde::Deserialize<'r>,
{
    let mut rows = stmt.query_and_then(params, |r| serde_rusqlite::from_row::<T>(r))?;
    let row = rows.next().ok_or("No row found")??;
    Ok(row)
}

pub fn fetch_rows<T>(stmt: &mut rusqlite::Statement, params: impl rusqlite::Params) -> Result<Vec<T>, Box<dyn Error>>
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
