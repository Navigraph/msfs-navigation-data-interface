use std::fs;
use std::io;
use std::path::PathBuf;

use crate::util;

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

    pub fn unzip_batch(&mut self, batch_size: usize) -> bool {
        if self.zip_archive.is_none() {
            return false;
        }
        let unwrapped_zip_archive = self.zip_archive.as_mut().unwrap();
        for _ in 0..batch_size {
            if self.current_file_index >= self.zip_file_count {
                // Done extracting, drop the zip archive
                self.zip_archive = None;
                return false;
            }

            let mut file = match unwrapped_zip_archive.by_index(self.current_file_index) {
                Ok(file) => file,
                Err(_) => continue,
            };
            let outpath = match file.enclosed_name() {
                Some(path) => self.path_buf.join(path),
                None => continue,
            };

            // Check how many times "." appears in the file name
            let dot_count = outpath.to_str().unwrap_or_default().matches('.').count();
            // Skip if there are more than 1 "." in the file name (MSFS crashes if we try to extract these files for some reason)
            if dot_count > 1 {
                self.current_file_index += 1;
                continue;
            }

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !util::path_exists(p) {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }
            self.current_file_index += 1;
        }
        true
    }
}
