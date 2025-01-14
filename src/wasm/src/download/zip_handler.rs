use std::{fs, io, path::PathBuf};

use crate::util;

#[derive(PartialEq, Eq)]

pub enum BatchReturn {
    MoreFilesToDelete,
    MoreFilesToUnzip,
    Finished,
}

pub struct ZipFileHandler<R: io::Read + io::Seek> {
    // Zip archive to extract
    pub zip_archive: Option<zip::ZipArchive<R>>,
    // Current file index in the zip archive
    pub current_file_index: usize,
    // Total number of files in the zip archive
    pub zip_file_count: usize,
    // Number of files deleted so far
    pub deleted: usize,
    // Path to the directory to extract to
    path_buf: PathBuf,
    // Whether or not we have cleaned the destination folder yet
    cleaned_destination: bool,
}

impl<R: io::Read + io::Seek> ZipFileHandler<R> {
    pub fn new(zip_archive: zip::ZipArchive<R>, path_buf: PathBuf) -> Self {
        // To make accessing zip archive size easier, we just store it to the struct instead of calling it every time
        // (avoids ownership issues)

        let zip_file_count = zip_archive.len();
        Self {
            zip_archive: Some(zip_archive),
            current_file_index: 0,
            zip_file_count,
            deleted: 0,
            path_buf,
            cleaned_destination: false,
        }
    }

    pub fn unzip_batch(
        &mut self,
        batch_size: usize,
    ) -> Result<BatchReturn, Box<dyn std::error::Error>> {
        if self.zip_archive.is_none() {
            return Err("No zip archive to extract".to_string().into());
        }

        // If we haven't cleaned the destination folder yet, do so now
        if !self.cleaned_destination {
            println!("[NAVIGRAPH] Cleaning Destination");
            util::delete_folder_recursively(&self.path_buf, Some(batch_size))?;
            if !util::path_exists(&self.path_buf) {
                fs::create_dir_all(&self.path_buf)?;
                self.cleaned_destination = true;
                return Ok(BatchReturn::MoreFilesToUnzip);
            }
            self.deleted += batch_size;
            return Ok(BatchReturn::MoreFilesToDelete);
        } else {
            println!("[NAVIGRAPH] Done Cleaning");
        }

        let zip_archive = self
            .zip_archive
            .as_mut()
            .ok_or_else(|| "Failed to access zip archive".to_string())?;

        for _ in 0..batch_size {
            if self.current_file_index >= self.zip_file_count {
                let fix_file = &self.path_buf.join("foo.txt");

                if !util::path_exists(&fix_file) {
                    fs::File::create(fix_file)?;
                }

                // Done extracting, drop the zip archive
                self.zip_archive = None;
                return Ok(BatchReturn::Finished);
            }

            let mut file = zip_archive.by_index(self.current_file_index)?;
            let outpath = self.path_buf.join(
                file.enclosed_name()
                    .ok_or_else(|| "Failed to get enclosed file name".to_string())?,
            );

            // Check how many times "." appears in the file name
            let dot_count = outpath
                .to_str()
                .ok_or_else(|| "Failed to convert path to string".to_string())?
                .matches('.')
                .count();

            // Skip if there are more than 1 "." in the file name (MSFS crashes if we try to extract these files for
            // some reason)
            if dot_count > 1 {
                self.current_file_index += 1;
                continue;
            }

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(outpath)
                    .map_err(|_| "Failed to create directory".to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !util::path_exists(p) {
                        fs::create_dir_all(p)
                            .map_err(|_| "Failed to create directory".to_string())?;
                    }
                }
                let mut outfile =
                    fs::File::create(outpath).map_err(|_| "Failed to create file".to_string())?;
                io::copy(&mut file, &mut outfile).map_err(|_| "Failed to copy file".to_string())?;
            }
            self.current_file_index += 1;
        }
        Ok(BatchReturn::MoreFilesToUnzip)
    }
}
