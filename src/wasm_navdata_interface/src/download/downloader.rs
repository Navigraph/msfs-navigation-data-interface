use std::cell::RefCell;

use std::io::Cursor;
use std::path::{PathBuf};
use std::rc::Rc;

use msfs::{network::*};

use crate::dispatcher::{Request, RequestStatus};
use crate::{
    download::zip_handler::{BatchReturn, ZipFileHandler},
};

pub struct DownloadOptions {
    batch_size: usize,
}

#[derive(PartialEq, Eq, Clone)]
pub enum DownloadStatus {
    NoDownload,
    Downloading,
    Extracting,
    Failed(String),
}

pub struct NavdataDownloader {
    zip_handler: RefCell<Option<ZipFileHandler<Cursor<Vec<u8>>>>>,
    status: RefCell<DownloadStatus>,
    options: RefCell<DownloadOptions>,
    request: RefCell<Option<Rc<RefCell<Request>>>>,
}

impl NavdataDownloader {
    pub fn new() -> Self {
        NavdataDownloader {
            zip_handler: RefCell::new(None),
            status: RefCell::new(DownloadStatus::NoDownload),
            options: RefCell::new(DownloadOptions { batch_size: 10 }), // default batch size
            request: RefCell::new(None),
        }
    }

    pub fn on_update(&self) {
        // Check if we failed and need to send an error message
        // We need to do this in its own variable since we can't borrow_mut and borrow at the same time (self.reset_download() borrows mutably)
        let failed_message = {
            let borrowed_status = self.status.borrow();
            if let DownloadStatus::Failed(ref message) = *borrowed_status {
                Some(message.clone())
            } else {
                None
            }
        };

        if let Some(message) = failed_message {
            // Send the error message to the JS side
            let borrowed_request = self.request.borrow();
            if (*borrowed_request).is_none() {
                println!("[WASM] Request is none");
                return;
            }
            let mut borrowed_request = borrowed_request.as_ref().unwrap().borrow_mut();
            borrowed_request.status = RequestStatus::Failure(message.clone());

            self.reset_download();
        }

        // Check if we are extracting
        // We need to do this in its own variable since we can't borrow_mut and borrow at the same time (self.unzip_batch() borrows mutably)
        let extract_next_batch = {
            let borrowed_zip_handler = self.zip_handler.borrow();
            if let Some(zip_handler) = borrowed_zip_handler.as_ref() {
                zip_handler.zip_file_count > zip_handler.current_file_index
            } else {
                // If there is no zip handler, we are not downloading and we don't need to do anything
                return;
            }
        };

        // Only proceed if there are zip files to process
        if extract_next_batch {
            // Unzip the next batch of files
            let unzip_status = self.unzip_batch(self.options.borrow().batch_size);
            match unzip_status {
                Ok(BatchReturn::Finished) => {
                    println!("[WASM] Finished extracting");
                    let borrowed_request = self.request.borrow();
                    if (*borrowed_request).is_none() {
                        println!("[WASM] Request is none");
                        return;
                    }
                    let mut borrowed_request = borrowed_request.as_ref().unwrap().borrow_mut();
                    borrowed_request.status = RequestStatus::Success(None);

                    self.reset_download();
                }
                Err(e) => {
                    println!("[WASM] Failed to unzip: {}", e);
                    let mut status = self.status.borrow_mut();
                    *status = DownloadStatus::Failed(format!("Failed to unzip: {}", e));
                }
                _ => (),
            }
        }
    }

    pub fn set_download_options(self: &Rc<Self>, request: Rc<RefCell<Request>>) {
        {
            let json = request.borrow().data.clone();
            // Get batch size, if it fails to parse then just return
            let batch_size = match json["batchSize"].as_u64() {
                Some(batch_size) => batch_size as usize,
                None => return,
            };

            // Set the options (only batch size for now)
            let mut options = self.options.borrow_mut();
            options.batch_size = batch_size;
        }
        request.borrow_mut().status = RequestStatus::Success(None);
    }

    pub fn download(self: &Rc<Self>, request: Rc<RefCell<Request>>) {
        // Silently fail if we are already downloading (maybe we should send an error message?)
        if *self.status.borrow() == DownloadStatus::Downloading {
            println!("[WASM] Already downloading");
            return;
        } else {
            // Set our status to downloading (needs to be done in its own scope so that the borrow_mut is dropped)
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Downloading;
            println!("[WASM] Downloading");
        }
        self.request.borrow_mut().replace(request.clone());

        let json = request.borrow().data.clone();

        let url = json["url"].as_str().unwrap_or_default().to_owned();

        // Check if json has "folder"
        let folder = json["folder"].as_str().unwrap_or_default().to_owned();

        // Make sure we have both a url and a folder
        if url.is_empty() || folder.is_empty() {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed("URL or folder is empty".to_string());
            return;
        }

        // Create the request
        let captured_self = self.clone();
        println!("[WASM] Creating request");
        match NetworkRequestBuilder::new(&url)
            .unwrap()
            .with_callback(move |network_request, status_code| {
                captured_self.request_finished_callback(network_request, status_code, folder)
            })
            .get()
        {
            Some(_) => (),
            None => {
                let mut status = self.status.borrow_mut();
                *status = DownloadStatus::Failed("Failed to create request".to_string());
            }
        }
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

        let path = PathBuf::from(format!("\\work/{}", folder));

        // Check the data from the request
        let data = request.data();
        if data.is_none() {
            let mut status = self.status.borrow_mut();
            *status = DownloadStatus::Failed("No data received".to_string());
            return;
        }
        // Extract the data from the request (safe to unwrap since we already checked if data was none)
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

    pub fn unzip_batch(
        &self,
        batch_size: usize,
    ) -> Result<BatchReturn, Box<dyn std::error::Error>> {
        let mut zip_handler = self.zip_handler.borrow_mut();

        let handler = zip_handler
            .as_mut()
            .ok_or_else(|| "Zip handler not found".to_string())?;
        let res = handler.unzip_batch(batch_size)?;

        Ok(res)
    }

    pub fn reset_download(&self) {
        // Use the take method to replace the current value with None and drop the old value.
        self.zip_handler.borrow_mut().take();

        *self.status.borrow_mut() = DownloadStatus::NoDownload;
    }
}
