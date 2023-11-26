use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use msfs::{commbus::*, network::*};

use crate::{download::zip_handler::ZipFileHandler, util};

pub struct DownloadOptions {
    batch_size: usize,
}

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
    options: RefCell<DownloadOptions>,
}

impl NavdataDownloader {
    pub fn new() -> Self {
        NavdataDownloader {
            zip_handler: RefCell::new(None),
            status: RefCell::new(DownloadStatus::NoDownload),
            options: RefCell::new(DownloadOptions { batch_size: 10 }), // default batch size
        }
    }

    pub fn on_update(&self) {
        let status = self.update_and_get_status();
        // If we are extracting, extract the next batch of files
        if status == DownloadStatus::Extracting {
            // Send the statistics to the JS side
            let statistics: DownloadStatistics = self.get_download_statistics().unwrap();
            let mut map = HashMap::new();
            map.insert("total", statistics.total_files);
            map.insert("unzipped", statistics.files_unzipped);
            let data = serde_json::to_string(&map).unwrap();
            CommBus::call(
                "NAVIGRAPH_UnzippedFilesRemaining",
                &data,
                CommBusBroadcastFlags::All,
            );

            // Unzip the next batch of files
            let has_more_files = self.unzip_batch(10);
            if !has_more_files {
                println!("[WASM] finished unzip");
                CommBus::call(
                    "NAVIGRAPH_NavdataDownloaded",
                    "",
                    CommBusBroadcastFlags::All,
                );

                self.clear_zip_handler();
            }
        } else if let DownloadStatus::Failed(_) = status {
            let error_message = match status {
                DownloadStatus::Failed(message) => message,
                _ => "Unknown error".to_owned(),
            };
            // Send the error message to the JS side
            let mut map = HashMap::new();
            map.insert("error", &error_message);
            let data = serde_json::to_string(&map).unwrap();
            CommBus::call(
                "NAVIGRAPH_DownloadFailed",
                &data,
                CommBusBroadcastFlags::All,
            );

            self.clear_zip_handler();
        }
    }

    pub fn set_download_options(self: &Rc<Self>, args: &str) {
        // Parse the JSON
        let json_result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(args);
        if json_result.is_err() {
            println!(
                "[WASM] Failed to parse JSON: {}",
                json_result.err().unwrap()
            );
            return;
        }
        let json = json_result.unwrap();
        let batch_size = json["batchSize"].as_u64().unwrap_or_default() as usize;

        // Set the options (only batch size for now)
        let mut options = self.options.borrow_mut();
        options.batch_size = batch_size;
    }

    pub fn download(self: &Rc<Self>, args: &str) {
        // Silently fail if we are already downloading (maybe we should send an error message?)
        if self.update_and_get_status() != DownloadStatus::NoDownload {
            println!("[WASM] Already downloading");
            return;
        }

        // Parse the JSON
        let json_result: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(args);
        if json_result.is_ok() {
            // Set our status to downloading (needs to be done in its own scope so that the borrow_mut is dropped)
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Downloading;
            println!("[WASM] Downloading");
        } else {
            // If we failed to parse the JSON, set our status to failed (read above for why this is in its own scope)
            let mut status = self.status.borrow_mut();
            let error = json_result.err().unwrap();
            *status = DownloadStatus::Failed(format!("JSON Parsing error from JS: {}", error));
            println!("[WASM] Failed: {}", error);
            return;
        }
        // Safe to unwrap since we already checked if it was an error
        let json = json_result.unwrap();
        let url = json["url"].as_str().unwrap_or_default();

        // Check if json has "folder"
        let folder = json["folder"].as_str().unwrap_or_default().to_owned();

        let captured_self = self.clone();
        NetworkRequestBuilder::new(url)
            .unwrap()
            .with_callback(move |request, status_code| {
                captured_self.request_finished_callback(request, status_code, folder)
            })
            .get()
            .unwrap();
    }

    fn request_finished_callback(&self, request: NetworkRequest, status_code: i32, folder: String) {
        // Fail if the status code is not 200
        if status_code != 200 {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed(format!(
                "Request failed with code {} and status {}",
                request.error_code(),
                status_code
            ));
            return;
        }

        let path = PathBuf::from(format!("\\work/navdata/{}", folder));
        // If the directory exists, delete it
        if util::path_exists(&path) {
            match util::delete_folder_recursively(&path) {
                Ok(_) => (),
                Err(e) => {
                    let mut status = self.status.borrow_mut();
                    *status = DownloadStatus::Failed(format!("Failed to delete directory: {}", e));
                    return;
                }
            }
        }
        // Re create the directory
        if let Err(e) = fs::create_dir_all(&path) {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed(format!("Failed to create directory: {}", e));
            return;
        }

        // Check the data from the request
        let data = request.data();
        if data.is_none() {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed("No data received".to_string());
            return;
        }
        // Extract the data from the request
        let data = data.unwrap();
        let cursor = Cursor::new(data);
        let zip = zip::ZipArchive::new(cursor);
        if zip.is_err() {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed(
                "Failed to create zip archive. Is this a zip file?".to_string(),
            );
            return;
        }
        // Unwrap is safe since we already checked if it was an error
        let zip = zip.unwrap();

        // Create the zip handler
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
        let zip_handler_ref = self.zip_handler.borrow();
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

    pub fn update_and_get_status(&self) -> DownloadStatus {
        // This basically either sets the status to no download, extracting, or done

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

    pub fn delete_all_navdata(&self) {
        let path = Path::new("\\work/navdata");
        if util::path_exists(path) {
            let _ = util::delete_folder_recursively(path);
        }
    }
}
