use std::fs;
use std::io;
use std::path::Path;

#[derive(PartialEq, Eq)]
pub enum PathType {
    File,
    Directory,
    DoesNotExist,
}

/// We aren't able to get file metadata in the sim so we can't use some of the standard library file system functions (like is_dir, exists, and some others)
pub fn get_path_type(path: &Path) -> PathType {
    let file_res = fs::File::open(path);
    if file_res.is_ok() {
        return PathType::File;
    }
    let mut dir_res = match fs::read_dir(path) {
        Ok(dir_res) => dir_res,
        Err(_) => {
            return PathType::DoesNotExist;
        }
    };

    let next = dir_res.next();

    if next.is_some() {
        // Safe to unwrap since we know next is some
        if next.unwrap().is_ok() {
            return PathType::Directory;
        }
    }
    PathType::DoesNotExist
}

pub fn path_exists(path: &Path) -> bool {
    get_path_type(path) != PathType::DoesNotExist
}

pub fn delete_folder_recursively(path: &Path) -> io::Result<()> {
    // Make sure we are deleting a directory (and in turn that it exists)
    if get_path_type(path) != PathType::Directory {
        return Ok(());
    }
    // We need to collect the entries into a vector since we can't iterate over them while deleting them
    for entry in fs::read_dir(path)?.collect::<Vec<_>>() {
        let entry = entry?;
        let path = entry.path();
        if get_path_type(&path) == PathType::Directory {
            delete_folder_recursively(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    fs::remove_dir(path)?;
    Ok(())
}
