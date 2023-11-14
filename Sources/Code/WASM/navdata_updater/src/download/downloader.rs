use std::cell::RefCell;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::rc::Rc;

use msfs::network::*;

use crate::download::zip_handler::ZipFileHandler;
use crate::util::JsonParser;

pub struct DownloadStatistics {
    pub total_files: usize,
    pub files_unzipped: usize,
    pub files_to_unzip: usize,
}

#[derive(PartialEq, Eq, Clone)]
pub enum DownloadStatus {
    NoDownload,
    Downloading,
    Extracting,
    Done,
    Failed(String),
}

pub struct NavdataDownloader {
    zip_handler: RefCell<Option<ZipFileHandler<Cursor<Vec<u8>>>>>,
    status: RefCell<DownloadStatus>,
}

impl NavdataDownloader {
    pub fn new() -> Self {
        NavdataDownloader {
            zip_handler: RefCell::new(None),
            status: RefCell::new(DownloadStatus::NoDownload),
        }
    }

    pub fn download(self: &Rc<Self>, args: &[u8]) {
        // Set our status to downloading (needs to be done in its own scope so that the borrow_mut is dropped)
        {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Downloading;
            println!("[WASM] Downloading");
        }

        let json_result = JsonParser::parse(args);
        if json_result.is_err() {
            let mut status = self.status.borrow_mut();
            let error = json_result.err().unwrap();
            *status = DownloadStatus::Failed(format!("JSON Parsing error from JS: {}", error));
            println!("[WASM] Failed: {}", error);
            return;
        }
        let json = json_result.unwrap();
        let url = json["url"].as_str().unwrap_or_default();

        let captured_self = self.clone();
        NetworkRequestBuilder::new(url)
            .unwrap()
            .with_callback(move |request, status_code| {
                captured_self.request_finished_callback(request, status_code)
            })
            .get()
            .unwrap();
    }

    fn request_finished_callback(&self, request: NetworkRequest, status_code: i32) {
        if status_code != 200 {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed(format!(
                "Request failed with code {} and status {}",
                request.error_code(),
                status_code
            ));
            return;
        }
        let path = PathBuf::from("\\work/navdata");
        if let Err(e) = fs::create_dir_all(&path) {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed(format!("Failed to create directory: {}", e));
            return;
        }

        let data = request.data();
        if data.is_none() {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed("No data received".to_string());
            return;
        }
        let data = data.unwrap();
        let cursor = Cursor::new(data);
        let zip = zip::ZipArchive::new(cursor).unwrap();

        let handler = ZipFileHandler::new(zip, path);

        let mut zip_handler = self.zip_handler.borrow_mut();
        *zip_handler = Some(handler);

        // Set our status to extracting (needs to be done in its own scope so that the borrow_mut is dropped)
        {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Extracting;
            println!("[WASM] Extracting");
        }
    }

    pub fn get_download_statistics(
        &self,
    ) -> Result<DownloadStatistics, Box<dyn std::error::Error>> {
        let zip_handler_ref = self.zip_handler.borrow(); // Borrow and hold onto the reference
        let zip_handler = zip_handler_ref.as_ref().ok_or("No zip handler")?;

        let total_files = zip_handler.zip_file_count;
        let files_unzipped = zip_handler.current_file_index;
        let files_to_unzip = total_files - files_unzipped;

        Ok(DownloadStatistics {
            total_files,
            files_unzipped,
            files_to_unzip,
        })
    }

    /// This basically either sets the status to no download, extracting, or done
    pub fn update_and_get_status(&self) -> DownloadStatus {
        let mut status = self.status.borrow_mut();
        let zip_handler_option = self.zip_handler.borrow();

        // If there is no zip handler, we are not downloading
        *status = match zip_handler_option.as_ref() {
            None => DownloadStatus::NoDownload,
            Some(zip_handler) => {
                // Downloaded all files
                match zip_handler
                    .zip_file_count
                    .cmp(&zip_handler.current_file_index)
                {
                    std::cmp::Ordering::Equal => DownloadStatus::Done,
                    std::cmp::Ordering::Greater => DownloadStatus::Extracting,
                    _ => return status.clone(),
                }
            }
        };

        // Clone here to return the updated status
        status.clone()
    }

    /// Unzips a batch of files
    ///
    /// Returns true if there are more files to unzip (false if we are done)
    pub fn unzip_batch(&self, batch_size: usize) -> bool {
        let mut zip_handler = self.zip_handler.borrow_mut();
        match zip_handler.as_mut() {
            Some(handler) => handler.unzip_batch(batch_size),
            None => false,
        }
    }

    pub fn clear_zip_handler(&self) {
        // Borrow mutably and set the zip handler to None. We need to do this in its own scope so that the borrow_mut is dropped
        // I really don't like this since update_and_get_status also borrows mutably but I don't know how else to do it/what the best way is
        {
            let mut zip_handler = self.zip_handler.borrow_mut();
            *zip_handler = None;
        }
        self.update_and_get_status();
    }
}
