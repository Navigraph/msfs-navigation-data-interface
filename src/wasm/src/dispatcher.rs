use std::{cell::RefCell, rc::Rc};

use msfs::{commbus::*, sys::sGaugeDrawData, MSFSEvent};

use crate::{
    download::downloader::NavdataDownloader,
    json_structs::{events, functions},
    query::database::Database,
    util,
};

#[derive(PartialEq, Eq)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Success(Option<serde_json::Value>),
    Failure(String),
}

pub struct Task {
    pub function_type: functions::FunctionType,
    pub id: String,
    pub data: Option<serde_json::Value>,
    pub status: TaskStatus,
}

impl Task {
    pub fn parse_data_as<T>(&self) -> Result<T, Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.data.clone().ok_or("No data provided")?;
        let params = serde_json::from_value::<T>(data)?;
        Ok(params)
    }
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
            delta_time: std::time::Duration::from_secs(u64::MAX), /* Initialize to max so that we send a heartbeat on
                                                                   * the first update */
            queue: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn on_msfs_event(&mut self, event: MSFSEvent) {
        match event {
            MSFSEvent::PostInitialize => {
                self.handle_initialized();
            },
            MSFSEvent::PreDraw(data) => {
                self.handle_update(data);
            },
            MSFSEvent::PreKill => {
                self.commbus.unregister_all();
            },

            _ => {},
        }
    }

    fn handle_initialized(&mut self) {
        {
            // We need to clone twice because we need to move the queue into the closure and then clone it again
            // whenever it gets called
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
            Dispatcher::send_event(events::EventType::Heartbeat, None);
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

            let function_type = task.borrow().function_type;

            match function_type {
                functions::FunctionType::DownloadNavdata => {
                    // We can't use the execute_task function here because the download process doesn't finish in the
                    // function call, which results in slightly "messier" code

                    // Close the database connection if it's open so we don't get any errors if we are replacing the
                    // database
                    self.database.close_connection();

                    // Now we can download the navdata
                    self.downloader.download(Rc::clone(task))
                },
                functions::FunctionType::SetDownloadOptions => {
                    Dispatcher::execute_task(task.clone(), |t| self.downloader.set_download_options(t))
                },
                functions::FunctionType::SetActiveDatabase => {
                    Dispatcher::execute_task(task.clone(), |t| self.database.set_active_database(t))
                },
                functions::FunctionType::ExecuteSQLQuery => {
                    Dispatcher::execute_task(task.clone(), |t| self.database.execute_sql_query(t))
                },
                functions::FunctionType::GetAirport => {
                    Dispatcher::execute_task(task.clone(), |t| self.database.get_airport(t))
                },
                functions::FunctionType::GetAirportsInRange => {
                    Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                        self.database.get_airports_in_range(t)
                    })
                },
                functions::FunctionType::GetAirways => {
                    Dispatcher::execute_task(task.clone(), |t| self.database.get_airways(t))
                },
                functions::FunctionType::GetAirwaysInRange => {
                    Dispatcher::execute_task(task.clone(), |t| self.database.get_airways_in_range(t))
                },
            }
        }

        // Process completed tasks (everything should at least be in progress at this point)
        queue.retain(|task| {
            if let TaskStatus::InProgress = task.borrow().status {
                return true;
            }

            let status: functions::FunctionStatus;
            let data: Option<serde_json::Value>;

            let (status, data) = match task.borrow().status {
                TaskStatus::Success(ref result) => {
                    status = functions::FunctionStatus::Success;
                    data = result.clone();
                    (status, data)
                },
                TaskStatus::Failure(ref error) => {
                    status = functions::FunctionStatus::Error;
                    data = Some(error.clone().into());
                    (status, data)
                },
                _ => unreachable!(), // This should never happen
            };

            let json = functions::FunctionResult {
                id: task.borrow().id.clone(),
                status,
                data,
            };

            if let Ok(serialized_json) = serde_json::to_string(&json) {
                CommBus::call("NAVIGRAPH_FunctionResult", &serialized_json, CommBusBroadcastFlags::All);
            }
            false
        });
    }

    /// Executes a task and handles the result (sets the status of the task)
    fn execute_task<F>(task: Rc<RefCell<Task>>, task_operation: F)
    where
        F: FnOnce(Rc<RefCell<Task>>) -> Result<(), Box<dyn std::error::Error>>,
    {
        match task_operation(task.clone()) {
            Ok(_) => (),
            Err(e) => {
                println!("Task failed: {}", e);
                task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            },
        }
    }

    fn add_to_queue(queue: Rc<RefCell<Vec<Rc<RefCell<Task>>>>>, args: &str) -> Result<(), Box<dyn std::error::Error>> {
        let args = util::trim_null_terminator(args);
        let json_result: functions::CallFunction = serde_json::from_str(args)?;

        queue.borrow_mut().push(Rc::new(RefCell::new(Task {
            function_type: json_result.function,
            id: json_result.id,
            data: json_result.data,
            status: TaskStatus::NotStarted,
        })));

        Ok(())
    }

    pub fn send_event(event: events::EventType, data: Option<serde_json::Value>) {
        let json = events::Event { event, data };
        if let Ok(serialized_json) = serde_json::to_string(&json) {
            CommBus::call("NAVIGRAPH_Event", &serialized_json, CommBusBroadcastFlags::All);
        } else {
            println!("Failed to serialize event");
        }
    }
}
