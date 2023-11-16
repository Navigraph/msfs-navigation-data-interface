use std::rc::Rc;

use crate::download::downloader::NavdataDownloader;
use msfs::{commbus::*, MSFSEvent};

pub struct Dispatcher<'a> {
    commbus: CommBus<'a>,
    downloader: Rc<NavdataDownloader>,
}

impl<'a> Dispatcher<'a> {
    pub fn new() -> Self {
        Dispatcher {
            commbus: CommBus::default(),
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
                self.commbus.unregister_all();
            }

            _ => {}
        }
    }

    fn handle_initialized(&mut self) {
        CommBus::call("NAVIGRAPH_Initialized", "", CommBusBroadcastFlags::All);
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
    }

    fn handle_update(&mut self) {
        // update unzip
        self.downloader.on_update();
    }

    fn trim_null_terminator(s: &str) -> &str {
        s.trim_end_matches(char::from(0))
    }
}
