use std::cell::RefCell;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::rc::Rc;

use msfs::network::*;

use crate::download::zip_handler::ZipFileHandler;
use crate::util::JsonParser;

pub struct NavdataDownloader {
    zip_handler: RefCell<Option<ZipFileHandler<Cursor<Vec<u8>>>>>,
}

impl NavdataDownloader {
    pub fn new() -> Self {
        NavdataDownloader {
            zip_handler: RefCell::new(None),
        }
    }

    pub fn download(self: &Rc<Self>, args: &[u8]) {
        println!("[WASM] call received");
        let json_result = JsonParser::parse(args);
        if json_result.is_err() {
            println!("[WASM] json error: {}", json_result.err().unwrap());
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
            println!("[WASM] request failed");
            return;
        }
        let path = PathBuf::from("\\work/navdata");
        if let Err(e) = fs::create_dir_all(&path) {
            println!("[WASM] directory error: {}", e);
            return;
        }

        let data = request.data().unwrap();
        let cursor = Cursor::new(data);
        let zip = zip::ZipArchive::new(cursor).unwrap();

        let handler = ZipFileHandler::new(zip, path);
        let mut zip_handler = self.zip_handler.borrow_mut();
        *zip_handler = Some(handler);
    }

    /// Returns the number of files left to unzip
    pub fn get_files_to_unzip(&self) -> usize {
        let zip_handler = self.zip_handler.borrow();
        match zip_handler.as_ref() {
            Some(handler) => handler.zip_file_count - handler.current_file_index,
            None => 0,
        }
    }

    /// Returns the total number of files in the zip
    pub fn get_total_files(&self) -> usize {
        let zip_handler = self.zip_handler.borrow();
        match zip_handler.as_ref() {
            Some(handler) => handler.zip_file_count,
            None => 0,
        }
    }

    /// Returns the number of files that have been unzipped
    pub fn get_files_unzipped(&self) -> usize {
        let zip_handler = self.zip_handler.borrow();
        match zip_handler.as_ref() {
            Some(handler) => handler.current_file_index,
            None => 0,
        }
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
}
