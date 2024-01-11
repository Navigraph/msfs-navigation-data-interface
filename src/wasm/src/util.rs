use std::{fs, io, path::Path};

use navigation_database::util::{get_path_type, PathType};

pub fn path_exists(path: &Path) -> bool { get_path_type(path) != PathType::DoesNotExist }

pub fn delete_folder_recursively(path: &Path, batch_size: Option<usize>) -> io::Result<()> {
    // Make sure we are deleting a directory (and in turn that it exists)
    if get_path_type(path) != PathType::Directory {
        return Ok(());
    }
    // Collect the entries that we will delete (taking into account the batch size)
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        entries.push(entry?);
        if let Some(batch_size) = batch_size {
            if entries.len() >= batch_size {
                break;
            }
        }
    }
    // After we have collected the entries, delete them
    for entry in entries {
        let path = entry.path();
        if get_path_type(&path) == PathType::Directory {
            delete_folder_recursively(&path, batch_size)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    // Check if the directory is empty. If it is, delete it
    let mut dir_res = fs::read_dir(path)?;
    let next = dir_res.next();
    if let Some(result) = next {
        if result.is_ok() {
            return Ok(());
        }
    } else {
        // Directory is empty, delete it
        fs::remove_dir(path)?;
    }
    Ok(())
}

pub fn trim_null_terminator(s: &str) -> &str { s.trim_end_matches(char::from(0)) }
