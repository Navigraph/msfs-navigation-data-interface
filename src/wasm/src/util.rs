use std::{fs, path::Path};

use anyhow::Result;

use navigation_database::util::{get_path_type, PathType};

pub fn path_exists(path: &Path) -> bool {
    get_path_type(path) != PathType::DoesNotExist
}

pub fn delete_folder_recursively(path: &Path, batch_size: Option<usize>) -> Result<()> {
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
        let path_type = get_path_type(&path);

        if path.file_name().unwrap() == "" {
            eprintln!("[NAVIGRAPH]: Bugged entry");
            continue;
        }

        if path_type == PathType::Directory {
            delete_folder_recursively(&path, batch_size)?;
        } else if path_type == PathType::File {
            fs::remove_file(&path)?;
        } else if path.extension().is_none() {
            // There are edge cases where completely empty directories are created and can't be deleted. They get
            // registered as "unknown" path type so we need to check if the path has an extension (which would tell us
            // if it's a file or a directory), and if it doesn't, we delete it as a directory
            let _ = fs::remove_dir(&path); // this can fail silently, but we don't care since there also might be cases
                                           // where a file literally doesn't exist
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

pub fn copy_files_to_folder(from: &Path, to: &Path) -> Result<()> {
    // Make sure we are copying a directory (and in turn that it exists)
    if get_path_type(from) != PathType::Directory {
        return Ok(());
    }
    // Let's clear the directory we are copying to
    delete_folder_recursively(to, None)?;
    // Create the directory we are copying to
    fs::create_dir(to)?;
    // Collect the entries that we will copy
    let entries = fs::read_dir(from)?;

    // Copy the entries
    for entry in entries {
        let Ok(entry) = entry else {
            eprintln!("[NAVIGRAPH]: Bugged entry");
            continue;
        };

        let path = entry.path();
        let path_type = get_path_type(&path);

        if path.file_name().unwrap() == "" {
            eprintln!("[NAVIGRAPH]: Bugged entry");
            continue;
        }

        if path_type == PathType::Directory {
            let new_dir = to.join(path.file_name().unwrap());
            fs::create_dir(&new_dir)?;
            copy_files_to_folder(&path, &new_dir)?;
        } else if path_type == PathType::File {
            fs::copy(&path, to.join(path.file_name().unwrap()))?;
        }
    }

    Ok(())
}

pub fn trim_null_terminator(s: &str) -> &str {
    s.trim_end_matches(char::from(0))
}
