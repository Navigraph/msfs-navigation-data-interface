use std::collections::HashMap;
use std::rc::Rc;

use crate::download::downloader::{DownloadStatus, NavdataDownloader};
use msfs::{commbus::*, MSFSEvent};

pub struct Dispatcher<'a> {
    commbus: CommBus<'a>,
    downloader: Rc<NavdataDownloader>,
}

impl<'a> Dispatcher<'a> {
    pub fn new() -> Self {
        Dispatcher {
            commbus: CommBus::new(),
            downloader: Rc::new(NavdataDownloader::new()),
        }
    }

    pub fn on_msfs_event(&mut self, event: MSFSEvent) {
        match event {
            MSFSEvent::PostInitialize => {
                self.handle_initialized();
            }
            MSFSEvent::PreUpdate => {
                self.handle_update();
            }
            MSFSEvent::PreKill => {
                // Drop commbus so that we in turn unregister the events. TODO wait for the unregister functions to be ported into the msfs-rs library
                CommBus::unregister_all();
            }

            _ => {}
        }
    }

    fn handle_initialized(&mut self) {
        CommBus::call("NAVIGRAPH_Initialized", "", CommBusBroadcastFlags::All);
        let captured_downloader = self.downloader.clone();
        self.commbus.register("NAVIGRAPH_DownloadNavdata", move |args| {
            captured_downloader.download(args)
        }).expect("Failed to register NAVIGRAPH_DownloadNavdata");
    }

    fn handle_update(&mut self) {
        // update unzip
        // todo: maybe another way to check instead of cloning? i mean we drop the value anyway but not sure on performance
        let captured_downloader = self.downloader.clone();
        let status = captured_downloader.update_and_get_status();
        if captured_downloader.update_and_get_status() == DownloadStatus::Extracting {
            let statistics = captured_downloader.get_download_statistics().unwrap(); // will always be Some because we are extracting
            let mut map = HashMap::new();
            map.insert("total", statistics.total_files);
            map.insert("unzipped", statistics.files_unzipped);
            let data = serde_json::to_string(&map).unwrap();
            CommBus::call(
                "NAVIGRAPH_UnzippedFilesRemaining",
                &data,
                CommBusBroadcastFlags::All,
            );
            let has_more_files = captured_downloader.unzip_batch(10);
            if !has_more_files {
                println!("[WASM] finished unzip");
                CommBus::call(
                    "NAVIGRAPH_NavdataDownloaded",
                    "",
                    CommBusBroadcastFlags::All,
                );
                captured_downloader.clear_zip_handler();
            }
        } else if let DownloadStatus::Failed(_) = status {
            let error_message = match status {
                DownloadStatus::Failed(message) => message,
                _ => "Unknown error".to_owned(),
            };
            let mut map = HashMap::new();
            map.insert("error", &error_message);
            let data = serde_json::to_string(&map).unwrap();
            CommBus::call(
                "NAVIGRAPH_DownloadFailed",
                &data,
                CommBusBroadcastFlags::All,
            );
            // clear the zip handler
            captured_downloader.clear_zip_handler();
        }
    }
}
