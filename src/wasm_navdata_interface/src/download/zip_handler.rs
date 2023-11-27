use std::fs;
use std::io;
use std::path::PathBuf;

use crate::util;

#[derive(PartialEq, Eq)]
pub enum BatchReturn {
    MoreFilesToUnzip,
    Finished,
}

pub struct ZipFileHandler<R: io::Read + io::Seek> {
    pub zip_archive: Option<zip::ZipArchive<R>>,
    path_buf: PathBuf,
    pub current_file_index: usize,
    pub zip_file_count: usize,
}

impl<R: io::Read + io::Seek> ZipFileHandler<R> {
    pub fn new(zip_archive: zip::ZipArchive<R>, path_buf: PathBuf) -> Self {
        // To make accessing zip archive size easier, we just store it to the struct instead of calling it every time (avoids ownership issues)
        let zip_file_count = zip_archive.len();
        Self {
            zip_archive: Some(zip_archive),
            path_buf,
            current_file_index: 0,
            zip_file_count,
        }
    }

    pub fn unzip_batch(
        &mut self,
        batch_size: usize,
    ) -> Result<BatchReturn, Box<dyn std::error::Error>> {
        if self.zip_archive.is_none() {
            return Err("No zip archive to extract".to_string().into());
        }

        let zip_archive = self
            .zip_archive
            .as_mut()
            .ok_or_else(|| "Failed to access zip archive".to_string())?;

        for _ in 0..batch_size {
            if self.current_file_index >= self.zip_file_count {
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

            // Skip if there are more than 1 "." in the file name (MSFS crashes if we try to extract these files for some reason)
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
