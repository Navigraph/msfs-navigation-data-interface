use std::{cell::RefCell, rc::Rc};

use crate::{download::downloader::NavdataDownloader, query::database::Database};
use msfs::{commbus::*, sys::sGaugeDrawData, MSFSEvent};

#[derive(Copy, Clone)]
pub enum RequestType {
    DownloadNavdata,
    SetDownloadOptions,
    DeleteAllNavdata,
    SetActiveDatabase,
    ExecuteSQLQuery,
}

impl RequestType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "DownloadNavdata" => Some(RequestType::DownloadNavdata),
            "SetDownloadOptions" => Some(RequestType::SetDownloadOptions),
            "DeleteAllNavdata" => Some(RequestType::DeleteAllNavdata),
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
    pub args: serde_json::Value,
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
            // TODO: why do we need to clone twice?
            let captured_queue = Rc::clone(&self.queue);
            self.commbus
                .register("NAVIGRAPH_CallFunction", move |args| {
                    // TODO: call fail
                    Dispatcher::add_to_queue(Rc::clone(&captured_queue), args);
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

        // Process queue
        let mut queue = self.queue.borrow_mut();
        if !queue.is_empty() {
            let requests_to_update: Vec<Rc<RefCell<Request>>> = queue
                .iter()
                .filter(|request| request.borrow().status == RequestStatus::NotStarted)
                .cloned()
                .collect();

            for request in requests_to_update {
                {
                    let mut mutable_request = request.borrow_mut();
                    mutable_request.status = RequestStatus::InProgress;
                }

                let request_type = {
                    let borrowed_request = request.borrow();
                    borrowed_request.request_type.clone()
                };
                match request_type {
                    RequestType::DownloadNavdata => {
                        self.downloader.download(Rc::clone(&request));
                    }
                    RequestType::SetDownloadOptions => {
                        self.downloader.set_download_options(Rc::clone(&request));
                    }
                    RequestType::DeleteAllNavdata => {
                        NavdataDownloader::delete_all_navdata(Rc::clone(&request));
                    }
                    RequestType::SetActiveDatabase => {
                        self.database.set_active_database(Rc::clone(&request));
                    }
                    RequestType::ExecuteSQLQuery => {
                        self.database.execute_sql_query(Rc::clone(&request));
                    }
                }
            }

            let collected_queue = queue.clone();
            for request in collected_queue.iter() {
                let borrowed_request = request.borrow();

                // At this point, every request should be in progress or completed (success or failure)
                if borrowed_request.status == RequestStatus::InProgress {
                    continue;
                }

                let mut json = serde_json::json!({
                    "id": borrowed_request.id,
                });
                if let RequestStatus::Success(ref request_data) = borrowed_request.status {
                    let request_data = match request_data {
                        Some(data) => data.clone(),
                        None => serde_json::json!({}),
                    };
                    println!("Request {} succeeded", borrowed_request.id);
                    json["status"] = "success".into();
                    json["data"] = request_data.into();
                } else if let RequestStatus::Failure(ref request_error) = borrowed_request.status {
                    println!("Request failed: {}", request_error);
                    json["status"] = "error".into();
                    json["message"] = request_error.clone().into();
                }
                if let Ok(serialized_json) = serde_json::to_string(&json) {
                    CommBus::call(
                        "NAVIGRAPH_FunctionResult",
                        &serialized_json,
                        CommBusBroadcastFlags::All,
                    );
                }
                queue.retain(|r| r.borrow().id != borrowed_request.id);
            }
        }

        self.downloader.on_update();
    }

    fn send_event(event: &str, data: Option<serde_json::Value>) {
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

    fn add_to_queue(
        queue: Rc<RefCell<Vec<Rc<RefCell<Request>>>>>,
        args: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let args = Dispatcher::trim_null_terminator(args);
        let json_result: serde_json::Value = serde_json::from_str(args)?;

        let request = json_result["function"]
            .as_str()
            .ok_or("Failed to parse function")?;
        let id = json_result["id"].as_str().ok_or("Failed to parse id")?;

        let request_type = RequestType::from_str(request).ok_or("Failed to parse request type")?;

        queue.borrow_mut().push(Rc::new(RefCell::new(Request {
            request_type: request_type,
            id: id.to_string(),
            args: json_result,
            status: RequestStatus::NotStarted,
        })));

        Ok(())
    }

    fn trim_null_terminator(s: &str) -> &str {
        s.trim_end_matches(char::from(0))
    }
}
