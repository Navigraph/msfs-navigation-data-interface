use std::{cell::RefCell, rc::Rc};

use crate::{download::downloader::NavdataDownloader, query::database::Database, util};
use msfs::{commbus::*, sys::sGaugeDrawData, MSFSEvent};

#[derive(Copy, Clone)]
pub enum TaskType {
    DownloadNavdata,
    SetDownloadOptions,
    SetActiveDatabase,
    ExecuteSQLQuery,
}

impl TaskType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "DownloadNavdata" => Some(TaskType::DownloadNavdata),
            "SetDownloadOptions" => Some(TaskType::SetDownloadOptions),
            "SetActiveDatabase" => Some(TaskType::SetActiveDatabase),
            "ExecuteSQLQuery" => Some(TaskType::ExecuteSQLQuery),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Success(Option<serde_json::Value>),
    Failure(String),
}

pub struct Task {
    pub task_type: TaskType,
    pub id: String,
    pub data: serde_json::Value,
    pub status: TaskStatus,
}

pub struct Dispatcher<'a> {
    commbus: CommBus<'a>,
    downloader: Rc<NavdataDownloader>,
    database: Rc<Database>,
    delta_time: std::time::Duration,
    queue: Rc<RefCell<Vec<Rc<RefCell<Task>>>>>,
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

        // Filter and update the status of the task that haven't started yet
        for task in queue
            .iter()
            .filter(|task| task.borrow().status == TaskStatus::NotStarted)
        {
            task.borrow_mut().status = TaskStatus::InProgress;

            let task_type = task.borrow().task_type;
            match task_type {
                TaskType::DownloadNavdata => self.downloader.download(Rc::clone(task)),
                TaskType::SetDownloadOptions => {
                    self.downloader.set_download_options(Rc::clone(task))
                }
                TaskType::SetActiveDatabase => self.database.set_active_database(Rc::clone(task)),
                TaskType::ExecuteSQLQuery => self.database.execute_sql_query(Rc::clone(task)),
            }
        }

        // Process completed tasks
        queue.retain(|task| {
            let borrowed_task = task.borrow();
            if borrowed_task.status == TaskStatus::InProgress {
                return true;
            }

            let mut json = serde_json::json!({ "id": borrowed_task.id });
            match borrowed_task.status {
                TaskStatus::Success(ref data) => {
                    println!("Task {} succeeded", borrowed_task.id);
                    json["status"] = "success".into();
                    json["data"] = data.clone().unwrap_or_else(|| serde_json::json!({}));
                }
                TaskStatus::Failure(ref error) => {
                    println!("Task failed: {}", error);
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
        queue: Rc<RefCell<Vec<Rc<RefCell<Task>>>>>,
        args: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let args = util::trim_null_terminator(args);
        let json_result: serde_json::Value = serde_json::from_str(args)?;

        let task = json_result["function"]
            .as_str()
            .ok_or("Failed to parse function")?;
        let id = json_result["id"].as_str().ok_or("Failed to parse id")?;

        let task_type = TaskType::from_str(task).ok_or("Failed to parse task type")?;

        queue.borrow_mut().push(Rc::new(RefCell::new(Task {
            task_type,
            id: id.to_string(),
            data: json_result["data"].clone(),
            status: TaskStatus::NotStarted,
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
