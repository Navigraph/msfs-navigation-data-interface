use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

use crate::download::downloader::NavdataDownloader;
use msfs::{commbus::*, MSFSEvent};

pub struct Dispatcher<'a> {
    commbus: Option<CommBus<'a>>,
    downloader: Rc<NavdataDownloader>,
}

impl<'a> Dispatcher<'a> {
    pub fn new() -> Self {
        Dispatcher {
            commbus: None,
            downloader: Rc::new(NavdataDownloader::new()),
        }
    }

    pub fn on_msfs_event(&mut self, event: MSFSEvent) {
        match event {
            MSFSEvent::PostInitialize => {
                self.handle_initialized();
            }
            MSFSEvent::PostUpdate => {
                self.handle_update();
            }
            MSFSEvent::PreKill => {
                // handle pre kill TODO wait for the unregister functions to be ported
            }

            _ => {}
        }
    }

    fn handle_initialized(&mut self) {
        println!("[WASM] Initialized");
        let captured_downloader = self.downloader.clone();
        self.commbus = CommBus::register("DownloadNavdata", move |args| {
            captured_downloader.download(args)
        });
    }

    fn handle_update(&mut self) {
        // update unzip
        // todo: maybe another way to check instead of cloning? i mean we drop the value anyway but not sure on performance
        let captured_downloader = self.downloader.clone();
        if captured_downloader.get_files_to_unzip() > 0 {
            let total_files = captured_downloader.get_total_files();
            let files_unzipped = captured_downloader.get_files_unzipped();
            let mut map = HashMap::new();
            map.insert("total", total_files);
            map.insert("unzipped", files_unzipped);
            let data = serde_json::to_string(&map).unwrap();
            // this is temporary until msfs-rs handles this unsafe stuff (soon TM)
            let i8_slice: &[i8] = unsafe { mem::transmute(data.as_bytes()) };
            println!(
                "[WASM] total: {}, unzipped: {}",
                total_files, files_unzipped
            );
            // only send the call if unzipped is divisible by 100 (kinda hacky but otherwise we flood the commbus (not good!))
            if files_unzipped % 100 == 0 {
                CommBus::call(
                    "UnzippedFilesRemaining",
                    i8_slice,
                    CommBusBroadcastFlags::JS,
                );
            }
            let has_more_files = captured_downloader.unzip_batch(10);
            if !has_more_files {
                println!("[WASM] finished unzip");
                CommBus::call("NavdataDownloaded", &[], CommBusBroadcastFlags::JS);
            }
        }
    }
}
