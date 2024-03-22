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
