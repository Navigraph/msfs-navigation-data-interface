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
    let dir_res = fs::read_dir(path);
    if dir_res.is_ok() {
        return PathType::Directory;
    }
    PathType::DoesNotExist
}
