use std::{cell::RefCell, rc::Rc};

use crate::{download::downloader::NavdataDownloader, query::database::Database, util};
use msfs::{commbus::*, sys::sGaugeDrawData, MSFSEvent};

#[derive(Copy, Clone)]
pub enum RequestType {
    DownloadNavdata,
    SetDownloadOptions,
    SetActiveDatabase,
    ExecuteSQLQuery,
}

impl RequestType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "DownloadNavdata" => Some(RequestType::DownloadNavdata),
            "SetDownloadOptions" => Some(RequestType::SetDownloadOptions),
            "SetActiveDatabase" => Some(RequestType::SetActiveDatabase),
            "ExecuteSQLQuery" => Some(RequestType::ExecuteSQLQuery),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum RequestStatus {
    NotStarted,
    InProgress,
    Success(Option<serde_json::Value>),
    Failure(String),
}

pub struct Request {
    pub request_type: RequestType,
    pub id: String,
    pub data: serde_json::Value,
    pub status: RequestStatus,
}

pub struct Dispatcher<'a> {
    commbus: CommBus<'a>,
    downloader: Rc<NavdataDownloader>,
    database: Rc<Database>,
    delta_time: std::time::Duration,
    queue: Rc<RefCell<Vec<Rc<RefCell<Request>>>>>,
}

impl<'a> Dispatcher<'a> {
    pub fn new() -> Self {
        Dispatcher {
            commbus: CommBus::default(),
            downloader: Rc::new(NavdataDownloader::new()),
            database: Rc::new(Database::new()),
            delta_time: std::time::Duration::from_secs(u64::MAX), // Initialize to max so that we send a heartbeat on the first update
            queue: Rc::new(RefCell::new(Vec::new())),
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
            // We need to clone twice because we need to move the queue into the closure and then clone it again whenever it gets called
            let captured_queue = Rc::clone(&self.queue);
            self.commbus
                .register("NAVIGRAPH_CallFunction", move |args| {
                    // TODO: maybe send error back to sim?
                    match Dispatcher::add_to_queue(Rc::clone(&captured_queue), args) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to add to queue: {}", e),
                    }
                })
                .expect("Failed to register NAVIGRAPH_CallFunction");
        }
    }

    fn handle_update(&mut self, data: &sGaugeDrawData) {
        // Accumulate delta time for heartbeat
        self.delta_time += data.delta_time();

        // Send heartbeat every 5 seconds
        if self.delta_time >= std::time::Duration::from_secs(5) {
            Dispatcher::send_event("Heartbeat", None);
            self.delta_time = std::time::Duration::from_secs(0);
        }

        self.process_queue();
        self.downloader.on_update();
    }

    fn process_queue(&mut self) {
        let mut queue = self.queue.borrow_mut();

        // Filter and update the status of the requests that haven't started yet
        for request in queue
            .iter()
            .filter(|request| request.borrow().status == RequestStatus::NotStarted)
        {
            request.borrow_mut().status = RequestStatus::InProgress;

            let request_type = request.borrow().request_type;
            match request_type {
                RequestType::DownloadNavdata => self.downloader.download(Rc::clone(request)),
                RequestType::SetDownloadOptions => {
                    self.downloader.set_download_options(Rc::clone(request))
                }
                RequestType::SetActiveDatabase => {
                    self.database.set_active_database(Rc::clone(request))
                }
                RequestType::ExecuteSQLQuery => {
                    self.database.execute_sql_query(Rc::clone(request))
                }
            }
        }

        // Process completed requests
        queue.retain(|request| {
            let borrowed_request = request.borrow();
            if borrowed_request.status == RequestStatus::InProgress {
                return true;
            }

            let mut json = serde_json::json!({ "id": borrowed_request.id });
            match borrowed_request.status {
                RequestStatus::Success(ref data) => {
                    println!("Request {} succeeded", borrowed_request.id);
                    json["status"] = "success".into();
                    json["data"] = data.clone().unwrap_or_else(|| serde_json::json!({}));
                }
                RequestStatus::Failure(ref error) => {
                    println!("Request failed: {}", error);
                    json["status"] = "error".into();
                    json["data"] = error.clone().into();
                }
                _ => (),
            }

            if let Ok(serialized_json) = serde_json::to_string(&json) {
                CommBus::call(
                    "NAVIGRAPH_FunctionResult",
                    &serialized_json,
                    CommBusBroadcastFlags::All,
                );
            }
            false
        });
    }

    fn add_to_queue(
        queue: Rc<RefCell<Vec<Rc<RefCell<Request>>>>>,
        args: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let args = util::trim_null_terminator(args);
        let json_result: serde_json::Value = serde_json::from_str(args)?;

        let request = json_result["function"]
            .as_str()
            .ok_or("Failed to parse function")?;
        let id = json_result["id"].as_str().ok_or("Failed to parse id")?;

        let request_type = RequestType::from_str(request).ok_or("Failed to parse request type")?;

        queue.borrow_mut().push(Rc::new(RefCell::new(Request {
            request_type,
            id: id.to_string(),
            data: json_result["data"].clone(),
            status: RequestStatus::NotStarted,
        })));

        Ok(())
    }

    pub fn send_event(event: &str, data: Option<serde_json::Value>) {
        // replace data with empty object if None
        let data = data.unwrap_or(serde_json::json!({}));
        let json = serde_json::json!({
            "event": event,
            "data": data,
        });
        if let Ok(serialized_json) = serde_json::to_string(&json) {
            CommBus::call(
                "NAVIGRAPH_Event",
                &serialized_json,
                CommBusBroadcastFlags::All,
            );
        } else {
            println!("Failed to serialize event {}", event);
        }
    }
}
