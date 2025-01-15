use std::{cell::RefCell, path::Path, rc::Rc};

use msfs::{commbus::*, network::NetworkRequestState, sys::sGaugeDrawData, MSFSEvent};
use navigation_database::database::Database;

use crate::{
    consts,
    download::downloader::{DownloadStatus, NavigationDataDownloader},
    json_structs::{
        events,
        functions::{CallFunction, FunctionResult, FunctionStatus, FunctionType},
        params,
    },
    meta::{self, InternalState},
    network_helper::NetworkHelper,
    util::{self, path_exists},
};

#[derive(PartialEq, Eq)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Success(Option<serde_json::Value>),
    Failure(String),
}

pub struct Task {
    pub function_type: FunctionType,
    pub id: String,
    pub data: Option<serde_json::Value>,
    pub status: TaskStatus,
    pub associated_network_request: Option<NetworkHelper>,
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
    downloader: Rc<NavigationDataDownloader>,
    database: Database,
    delta_time: std::time::Duration,
    queue: Rc<RefCell<Vec<Rc<RefCell<Task>>>>>,
}

impl<'a> Dispatcher<'a> {
    pub fn new() -> Self {
        Dispatcher {
            commbus: CommBus::default(),
            downloader: Rc::new(NavigationDataDownloader::new()),
            database: Database::new(),
            delta_time: std::time::Duration::from_secs(u64::MAX), /* Initialize to max so that we send a heartbeat on
                                                                   * the first update */
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
        self.load_database();
        // We need to clone twice because we need to move the queue into the closure and then clone it again
        // whenever it gets called
        let captured_queue = Rc::clone(&self.queue);
        self.commbus
            .register("NAVIGRAPH_CallFunction", move |args| {
                // TODO: maybe send error back to sim?
                match Dispatcher::add_to_queue(Rc::clone(&captured_queue), args) {
                    Ok(_) => (),
                    Err(e) => println!("[NAVIGRAPH] Failed to add to queue: {}", e),
                }
            })
            .expect("Failed to register NAVIGRAPH_CallFunction");
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

        // Because the download process doesn't finish in the function call, we need to check if the download is finished to call the on_download_finish function
        if *self.downloader.download_status.borrow() == DownloadStatus::Downloaded {
            self.on_download_finish();
            self.downloader.acknowledge_download();
        }
    }
    fn load_database(&mut self) {
        println!("[NAVIGRAPH] Loading database");

        // Go through logic to determine which database to load

        // Are we bundled? None means we haven't installed anything yet
        let is_bundled = meta::get_internal_state()
            .map(|internal_state| Some(internal_state.is_bundled))
            .unwrap_or(None);

        // Get the installed cycle (if it exists)
        let installed_cycle = match meta::get_installed_cycle_from_json(
            &Path::new(consts::NAVIGATION_DATA_WORK_LOCATION).join("cycle.json"),
        ) {
            Ok(cycle) => Some(cycle.cycle),
            Err(_) => None,
        };

        // Get the bundled cycle (if it exists)
        let bundled_cycle = match meta::get_installed_cycle_from_json(
            &Path::new(consts::NAVIGATION_DATA_DEFAULT_LOCATION).join("cycle.json"),
        ) {
            Ok(cycle) => Some(cycle.cycle),
            Err(_) => None,
        };

        // Determine if we are bundled ONLY and the bundled cycle is newer than the installed (old bundled) cycle
        let bundled_updated = if is_bundled.is_some_and(|x| x) {
            // Clippy yells but this isn't good to switch until if-let chaining is implemented (Rust 2024)
            if installed_cycle.is_some() && bundled_cycle.is_some() {
                bundled_cycle.unwrap() > installed_cycle.unwrap()
            } else {
                false
            }
        } else {
            false
        };

        // If there is no addon config, we can assume that we need to copy the bundled database to the work location
        let need_to_copy = is_bundled.is_none();

        // If we are bundled and the installed cycle is older than the bundled cycle, we need to copy the bundled database to the work location. Or if we haven't installed anything yet, we need to copy the bundled database to the work location
        if bundled_updated || need_to_copy {
            match util::copy_files_to_folder(
                Path::new(consts::NAVIGATION_DATA_DEFAULT_LOCATION),
                Path::new(consts::NAVIGATION_DATA_WORK_LOCATION),
            ) {
                Ok(_) => {
                    // Set the internal state to bundled
                    let res = meta::set_internal_state(InternalState { is_bundled: true });
                    if let Err(e) = res {
                        println!("[NAVIGRAPH] Failed to set internal state: {}", e);
                    }
                }
                Err(e) => {
                    println!(
                        "[NAVIGRAPH] Failed to copy database from default location to work location: {}",
                        e
                    );
                    return;
                }
            }
        }

        // Finally, set the active database
        if path_exists(Path::new(consts::NAVIGATION_DATA_WORK_LOCATION)) {
            match self
                .database
                .set_active_database(consts::NAVIGATION_DATA_WORK_LOCATION.to_owned())
            {
                Ok(_) => {
                    println!("[NAVIGRAPH] Loaded database");
                }
                Err(e) => {
                    println!("[NAVIGRAPH] Failed to load database: {}", e);
                }
            }
        } else {
            println!("[NAVIGRAPH] Failed to load database: there is no installed database");
        }
    }

    fn on_download_finish(&mut self) {
        if let Ok(path) =
            navigation_database::util::find_sqlite_file(consts::NAVIGATION_DATA_WORK_LOCATION)
        {
            match self.database.set_active_database(path) {
                Ok(_) => {}
                Err(e) => {
                    println!("[NAVIGRAPH] Failed to set active database: {}", e);
                }
            };
        }
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
                FunctionType::DownloadNavigationData => {
                    // We can't use the execute_task function here because the download process doesn't finish in the
                    // function call, which results in slightly "messier" code

                    // Close the database connection if it's open so we don't get any errors if we are replacing the
                    // database
                    self.database.close_connection();

                    // Now we can download the navigation data
                    self.downloader.download(Rc::clone(task));
                }
                FunctionType::SetDownloadOptions => Dispatcher::execute_task(task.clone(), |t| {
                    self.downloader.set_download_options(t)
                }),
                FunctionType::GetNavigationDataInstallStatus => {
                    // We can't use the execute_task function here because the download process doesn't finish in the
                    // function call, which results in slightly "messier" code

                    // We first need to initialize the network request and then wait for the response
                    meta::start_network_request(Rc::clone(task))
                }
                FunctionType::ExecuteSQLQuery => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t
                        .borrow()
                        .parse_data_as::<params::ExecuteSQLQueryParams>()?;
                    let data = self.database.execute_sql_query(params.sql, params.params)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(data));

                    Ok(())
                }),
                FunctionType::GetDatabaseInfo => Dispatcher::execute_task(task.clone(), |t| {
                    let info = self.database.get_database_info()?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(info)?));

                    Ok(())
                }),
                FunctionType::GetAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let airport = self.database.get_airport(params.ident)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(airport)?));

                    Ok(())
                }),
                FunctionType::GetWaypoints => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let waypoints = self.database.get_waypoints(params.ident)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(waypoints)?));

                    Ok(())
                }),
                FunctionType::GetVhfNavaids => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let vhf_navaids = self.database.get_vhf_navaids(params.ident)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(vhf_navaids)?));

                    Ok(())
                }),
                FunctionType::GetNdbNavaids => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let ndb_navaids = self.database.get_ndb_navaids(params.ident)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(ndb_navaids)?));

                    Ok(())
                }),
                FunctionType::GetAirways => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let airways = self.database.get_airways(params.ident)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(airways)?));

                    Ok(())
                }),
                FunctionType::GetAirwaysAtFix => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtFixParams>()?;
                    let airways = self
                        .database
                        .get_airways_at_fix(params.fix_ident, params.fix_icao_code)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(airways)?));

                    Ok(())
                }),
                FunctionType::GetAirportsInRange => {
                    Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                        let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                        let airports = self
                            .database
                            .get_airports_in_range(params.center, params.range)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(airports)?));

                        Ok(())
                    })
                }
                FunctionType::GetWaypointsInRange => {
                    Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                        let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                        let waypoints = self
                            .database
                            .get_waypoints_in_range(params.center, params.range)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(waypoints)?));

                        Ok(())
                    })
                }
                FunctionType::GetVhfNavaidsInRange => {
                    Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                        let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                        let navaids = self
                            .database
                            .get_vhf_navaids_in_range(params.center, params.range)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(navaids)?));

                        Ok(())
                    })
                }
                FunctionType::GetNdbNavaidsInRange => {
                    Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                        let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                        let navaids = self
                            .database
                            .get_ndb_navaids_in_range(params.center, params.range)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(navaids)?));

                        Ok(())
                    })
                }
                FunctionType::GetAirwaysInRange => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let airways = self
                        .database
                        .get_airways_in_range(params.center, params.range)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(airways)?));

                    Ok(())
                }),
                FunctionType::GetControlledAirspacesInRange => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                        let airspaces = self
                            .database
                            .get_controlled_airspaces_in_range(params.center, params.range)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(airspaces)?));

                        Ok(())
                    })
                }
                FunctionType::GetRestrictiveAirspacesInRange => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                        let airspaces = self
                            .database
                            .get_restrictive_airspaces_in_range(params.center, params.range)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(airspaces)?));

                        Ok(())
                    })
                }
                FunctionType::GetCommunicationsInRange => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                        let communications = self
                            .database
                            .get_communications_in_range(params.center, params.range)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(communications)?));

                        Ok(())
                    })
                }
                FunctionType::GetRunwaysAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let runways = self.database.get_runways_at_airport(params.airport_ident)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(runways)?));

                    Ok(())
                }),
                FunctionType::GetDeparturesAtAirport => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                        let departures = self
                            .database
                            .get_departures_at_airport(params.airport_ident)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(departures)?));

                        Ok(())
                    })
                }
                FunctionType::GetArrivalsAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let arrivals = self
                        .database
                        .get_arrivals_at_airport(params.airport_ident)?;

                    t.borrow_mut().status =
                        TaskStatus::Success(Some(serde_json::to_value(arrivals)?));

                    Ok(())
                }),
                FunctionType::GetApproachesAtAirport => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                        let arrivals = self
                            .database
                            .get_approaches_at_airport(params.airport_ident)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(arrivals)?));

                        Ok(())
                    })
                }
                FunctionType::GetWaypointsAtAirport => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                        let waypoints = self
                            .database
                            .get_waypoints_at_airport(params.airport_ident)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(waypoints)?));

                        Ok(())
                    })
                }
                FunctionType::GetNdbNavaidsAtAirport => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                        let navaids = self
                            .database
                            .get_ndb_navaids_at_airport(params.airport_ident)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(navaids)?));

                        Ok(())
                    })
                }
                FunctionType::GetGatesAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let gates = self.database.get_gates_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(gates)?));

                    Ok(())
                }),
                FunctionType::GetCommunicationsAtAirport => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                        let communications = self
                            .database
                            .get_communications_at_airport(params.airport_ident)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(communications)?));

                        Ok(())
                    })
                }
                FunctionType::GetGlsNavaidsAtAirport => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                        let navaids = self
                            .database
                            .get_gls_navaids_at_airport(params.airport_ident)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(navaids)?));

                        Ok(())
                    })
                }
                FunctionType::GetPathPointsAtAirport => {
                    Dispatcher::execute_task(task.clone(), |t| {
                        let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                        let pathpoints = self
                            .database
                            .get_path_points_at_airport(params.airport_ident)?;

                        t.borrow_mut().status =
                            TaskStatus::Success(Some(serde_json::to_value(pathpoints)?));

                        Ok(())
                    })
                }
            }
        }

        // Network request tasks
        for task in queue
            .iter()
            .filter(|task| task.borrow().status == TaskStatus::InProgress)
        {
            let response_state = match task.borrow().associated_network_request {
                Some(ref request) => request.response_state(),
                None => continue,
            };
            let function_type = task.borrow().function_type;
            if response_state == NetworkRequestState::DataReady {
                match function_type {
                    FunctionType::GetNavigationDataInstallStatus => {
                        println!("[NAVIGRAPH] Network request completed, getting install status");
                        meta::get_navigation_data_install_status(Rc::clone(task));
                        println!("[NAVIGRAPH] Install status task completed");
                    }
                    _ => {
                        // Should not happen for now
                        println!("[NAVIGRAPH] Network request completed but no handler for this type of request");
                    }
                }
            } else if response_state == NetworkRequestState::Failed {
                task.borrow_mut().status = TaskStatus::Failure("Network request failed".to_owned());
            }
        }

        // Process completed tasks (everything should at least be in progress at this point)
        queue.retain(|task| {
            if let TaskStatus::InProgress = task.borrow().status {
                return true;
            }

            let status: FunctionStatus;
            let data: Option<serde_json::Value>;

            let (status, data) = match task.borrow().status {
                TaskStatus::Success(ref result) => {
                    status = FunctionStatus::Success;
                    data = result.clone();
                    (status, data)
                }
                TaskStatus::Failure(ref error) => {
                    status = FunctionStatus::Error;
                    data = Some(error.clone().into());
                    (status, data)
                }
                _ => unreachable!(), // This should never happen
            };

            let json = FunctionResult {
                id: task.borrow().id.clone(),
                status,
                data,
            };

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

    /// Executes a task and handles the result (sets the status of the task)
    fn execute_task<F>(task: Rc<RefCell<Task>>, task_operation: F)
    where
        F: FnOnce(Rc<RefCell<Task>>) -> Result<(), Box<dyn std::error::Error>>,
    {
        match task_operation(task.clone()) {
            Ok(_) => (),
            Err(e) => {
                println!("[NAVIGRAPH] Task failed: {}", e);
                task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            }
        }
    }

    fn add_to_queue(
        queue: Rc<RefCell<Vec<Rc<RefCell<Task>>>>>,
        args: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let args = util::trim_null_terminator(args);
        let json_result: CallFunction = serde_json::from_str(args)?;

        queue.borrow_mut().push(Rc::new(RefCell::new(Task {
            function_type: json_result.function,
            id: json_result.id,
            data: json_result.data,
            status: TaskStatus::NotStarted,
            associated_network_request: None,
        })));

        Ok(())
    }

    pub fn send_event(event: events::EventType, data: Option<serde_json::Value>) {
        let json = events::Event { event, data };

        if let Ok(serialized_json) = serde_json::to_string(&json) {
            CommBus::call(
                "NAVIGRAPH_Event",
                &serialized_json,
                CommBusBroadcastFlags::All,
            );
        } else {
            println!("[NAVIGRAPH] Failed to serialize event");
        }
    }
}
