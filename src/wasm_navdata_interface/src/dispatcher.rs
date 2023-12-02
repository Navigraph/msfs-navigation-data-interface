use std::rc::Rc;

use crate::download::downloader::NavdataDownloader;
use msfs::{commbus::*, sys::sGaugeDrawData, MSFSEvent};

pub struct Dispatcher<'a> {
    commbus: CommBus<'a>,
    downloader: Rc<NavdataDownloader>,
    delta_time: std::time::Duration,
}

impl<'a> Dispatcher<'a> {
    pub fn new() -> Self {
        Dispatcher {
            commbus: CommBus::default(),
            downloader: Rc::new(NavdataDownloader::new()),
            delta_time: std::time::Duration::from_secs(0),
        }
    }

    pub fn on_msfs_event(&mut self, event: MSFSEvent) {
        match event {
            MSFSEvent::PostInitialize => {
                self.handle_initialized();
            }
            MSFSEvent::PreDraw(data) => {
                self.handle_update(data);
            }
            MSFSEvent::PreKill => {
                self.commbus.unregister_all();
            }

            _ => {}
        }
    }

    fn handle_initialized(&mut self) {
        {
            let captured_downloader = self.downloader.clone();
            self.commbus
                .register("NAVIGRAPH_DownloadNavdata", move |args| {
                    captured_downloader.download(Dispatcher::trim_null_terminator(args))
                })
                .expect("Failed to register NAVIGRAPH_DownloadNavdata");
        }
        {
            let captured_downloader = self.downloader.clone();
            self.commbus
                .register("NAVIGRAPH_SetDownloadOptions", move |args| {
                    captured_downloader.set_download_options(Dispatcher::trim_null_terminator(args))
                })
                .expect("Failed to register NAVIGRAPH_SetDownloadOptions");
        }
        {
            let captured_downloader = self.downloader.clone();
            self.commbus
                .register("NAVIGRAPH_DeleteAllNavdata", move |_| {
                    captured_downloader.delete_all_navdata()
                })
                .expect("Failed to register NAVIGRAPH_DeleteAllNavdata");
        }
    }

    fn handle_update(&mut self, data: &sGaugeDrawData) {
        // Accumulate delta time for heartbeat
        self.delta_time += data.delta_time();

        // Send heartbeat every 5 seconds
        if self.delta_time >= std::time::Duration::from_secs(5) {
            CommBus::call("NAVIGRAPH_Heartbeat", "", CommBusBroadcastFlags::All);
            self.delta_time = std::time::Duration::from_secs(0);
        }

        self.downloader.on_update();
    }

    fn trim_null_terminator(s: &str) -> &str {
        s.trim_end_matches(char::from(0))
    }
}
