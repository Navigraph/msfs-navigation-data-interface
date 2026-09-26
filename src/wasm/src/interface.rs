use std::{collections::VecDeque, time::Instant};

use anyhow::Result;
use sentry::{integrations::anyhow::capture_anyhow, protocol::Context};
use serde::Serialize;

use crate::{
    funcs::{InterfaceFunction, RunStatus},
    platform::{ActivePlatform, Platform},
};

/// Amount of MS between dispatches of the heartbeat event
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
pub struct InterfaceEvent {
    event: NavigraphEventType,
    data: Option<serde_json::Value>,
}

impl InterfaceEvent {
    /// Send a heartbeat event
    pub fn send_heartbeat() -> Result<()> {
        let event = Self {
            event: NavigraphEventType::Heartbeat,
            data: None,
        };

        let serialized = serde_json::to_string(&event)?;

        ActivePlatform::send_message("NAVIGRAPH_Event", &serialized);

        Ok(())
    }

    /// Send a download progress event
    ///
    /// * `event` - The download progress event data
    pub fn send_download_progress_event(event: DownloadProgressEvent) -> Result<()> {
        let event = Self {
            event: NavigraphEventType::DownloadProgress,
            data: Some(serde_json::to_value(event)?),
        };

        let serialized = serde_json::to_string(&event)?;

        ActivePlatform::send_message("NAVIGRAPH_Event", &serialized);

        Ok(())
    }
}

/// The queue of called functions, which are run in order, plus the heartbeat
#[derive(Default)]
pub struct FunctionQueue {
    queue: VecDeque<InterfaceFunction>,
    last_heartbeat: Option<Instant>,
}

impl FunctionQueue {
    /// Queue a function call
    ///
    /// * `args` - The `NAVIGRAPH_CallFunction` payload
    pub fn push(&mut self, args: &str) {
        // Parse the message as a function. We need to trim off the null terminator at the end
        let params = match serde_json::from_str::<InterfaceFunction>(
            args.trim_end_matches(char::from(0)),
        ) {
            Ok(p) => p,
            Err(e) => {
                sentry::capture_message(
                    &format!("Unable to parse InterfaceFunction from {args} due to error {e}",),
                    sentry::Level::Warning,
                );
                return;
            }
        };

        // Finally, push the function into our queue
        self.queue.push_back(params);
    }

    /// Run the queued functions, and send the heartbeat if due
    pub fn update(&mut self) -> Result<()> {
        // Process one function at a time. If the function returns InProgress, don't continue on to the next item in order to preserve call order
        while let Some(function) = self.queue.front_mut() {
            match function.run() {
                Ok(RunStatus::InProgress) => break,
                Ok(RunStatus::Finished) => {
                    self.queue.pop_front();
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
                    self.queue.pop_front();
                }
            };
        }

        self.heartbeat()
    }

    /// Send a heartbeat if none has been sent yet, or if we have passed the interval
    pub fn heartbeat(&mut self) -> Result<()> {
        let due = self
            .last_heartbeat
            .map_or(true, |last| last.elapsed().as_millis() >= HEARTBEAT_INTERVAL_MS);

        if due {
            InterfaceEvent::send_heartbeat()?;
            self.last_heartbeat = Some(Instant::now());
        }

        Ok(())
    }
}
