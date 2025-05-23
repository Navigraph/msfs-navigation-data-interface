use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Instant};

use anyhow::{anyhow, Result};
use funcs::{InterfaceFunction, RunStatus};
use msfs::commbus::{CommBus, CommBusBroadcastFlags};
use sentry::{integrations::anyhow::capture_anyhow, protocol::Context};
use sentry_gauge::{wrap_gauge_with_sentry, SentryGauge};
use serde::Serialize;

/// Developer-configured values for interface
mod config;
/// SQLite mapping implementation
mod database;
/// Interface function definitions
mod funcs;
/// Futures implementations for use in interface functions
mod futures;
/// The sentry wrapper implementation around the MSFS gauge callbacks
mod sentry_gauge;

/// Amount of MS between dispatches of the heartbeat commbus event
const HEARTBEAT_INTERVAL_MS: u128 = 1000;

/// The data associated with the `DownloadProgress` event
#[derive(Serialize)]
pub struct DownloadProgressEvent {
    /// The total amount of bytes to download
    pub total_bytes: usize,
    /// The amount of bytes downloaded
    pub downloaded_bytes: usize,
    /// The chunk number (starting at 0) of the current download
    pub current_chunk: usize,
    /// The total number of chunks needed to download
    pub total_chunks: usize,
}

/// The types of events that can be emitted from the interface
#[derive(Serialize)]
enum NavigraphEventType {
    Heartbeat,
    DownloadProgress,
}

/// The structure of an event message
#[derive(Serialize)]
struct InterfaceEvent {
    event: NavigraphEventType,
    data: Option<serde_json::Value>,
}

impl InterfaceEvent {
    /// Send a heartbeat event across the commbus
    pub fn send_heartbeat() -> Result<()> {
        let event = Self {
            event: NavigraphEventType::Heartbeat,
            data: None,
        };

        let serialized = serde_json::to_string(&event)?;

        CommBus::call("NAVIGRAPH_Event", &serialized, CommBusBroadcastFlags::All);

        Ok(())
    }

    /// Send a download progress event across the commbus
    ///
    /// * `event` - The download progress event data
    pub fn send_download_progress_event(event: DownloadProgressEvent) -> Result<()> {
        let event = Self {
            event: NavigraphEventType::DownloadProgress,
            data: Some(serde_json::to_value(event)?),
        };

        let serialized = serde_json::to_string(&event)?;

        CommBus::call("NAVIGRAPH_Event", &serialized, CommBusBroadcastFlags::All);

        Ok(())
    }
}

/// The main state for the interface
struct NavigationDataInterface<'a> {
    _commbus: CommBus<'a>,
    processing_queue: Rc<RefCell<VecDeque<InterfaceFunction>>>,
    last_heartbeat: Instant,
}

impl SentryGauge for NavigationDataInterface<'_> {
    fn initialize() -> Result<Self>
    where
        Self: Sized,
    {
        // Initialize commbus
        let mut commbus = CommBus::default();
        let processing_queue = Rc::new(RefCell::new(VecDeque::new()));

        // Create the NAVIGRAPH_CallFunction callback
        let processing_queue_clone = Rc::clone(&processing_queue);
        commbus
            .register("NAVIGRAPH_CallFunction", move |args| {
                // Try to get the queue
                let Ok(mut processing_queue) = processing_queue_clone.try_borrow_mut() else {
                    sentry::capture_message(
                        "Unable to borrow processing queue",
                        sentry::Level::Warning,
                    );
                    return;
                };

                // Parse the message as a function. We need to trim off the null terminator at the end
                let params = match serde_json::from_str::<InterfaceFunction>(
                    args.trim_end_matches(char::from(0)),
                ) {
                    Ok(p) => p,
                    Err(e) => {
                        sentry::capture_message(
                            &format!(
                                "Unable to parse InterfaceFunction from {args} due to error {e}",
                            ),
                            sentry::Level::Warning,
                        );
                        return;
                    }
                };

                // Finally, push the function into our queue
                processing_queue.push_back(params);
            })
            .ok_or(anyhow!("Unable to register NAVIGRAPH_CallFunction"))?;

        // Send first heartbeat
        let last_heartbeat = Instant::now();
        InterfaceEvent::send_heartbeat()?;

        Ok(Self {
            _commbus: commbus,
            processing_queue,
            last_heartbeat,
        })
    }

    fn update(&mut self) -> Result<()> {
        let mut queue = self.processing_queue.try_borrow_mut()?;

        // Process one function at a time. If the function returns InProgress, don't continue on to the next item in order to preserve call order
        while let Some(function) = queue.front_mut() {
            match function.run() {
                Ok(RunStatus::InProgress) => break,
                Ok(RunStatus::Finished) => {
                    queue.pop_front();
                }
                Err(e) => {
                    // Report error
                    sentry::with_scope(
                        |scope| {
                            scope.set_context(
                                "Interface Function",
                                Context::Other(function.get_function_details()),
                            );
                        },
                        || capture_anyhow(&e),
                    );
                    println!("[NAVIGRAPH]: Error occurred in function execution: {e}");
                    // Remove item
                    queue.pop_front();
                }
            };
        }

        // Send heartbeat if we have passed the interval
        if self.last_heartbeat.elapsed().as_millis() >= HEARTBEAT_INTERVAL_MS {
            InterfaceEvent::send_heartbeat()?;
            self.last_heartbeat = Instant::now();
        }

        Ok(())
    }
}

sentry_gauge!(NavigationDataInterface, navigation_data_interface);
