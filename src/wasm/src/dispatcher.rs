use std::{
    cell::RefCell,
    error::Error,
    fs,
    io::{self},
    path::Path,
    rc::Rc,
};

use msfs::{commbus::*, network::NetworkRequestState, sys::sGaugeDrawData, MSFSEvent};
use navigation_database::{
    database::DatabaseV1,
    enums::InterfaceFormat,
    manual::database::DatabaseManual,
    traits::{DatabaseEnum, DatabaseTrait, InstalledNavigationDataCycleInfo, PackageInfo},
    v2::database::DatabaseV2,
};

use crate::{
    consts,
    download::downloader::{DownloadStatus, NavigationDataDownloader},
    json_structs::{
        events,
        functions::{CallFunction, FunctionResult, FunctionStatus, FunctionType},
        params,
    },
    meta::{self},
    network_helper::NetworkHelper,
    util::{self, generate_uuid_from_cycle, generate_uuid_from_path, path_exists},
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
    database: RefCell<DatabaseEnum>,
    delta_time: std::time::Duration,
    queue: Rc<RefCell<Vec<Rc<RefCell<Task>>>>>,
    set_active_on_finish: RefCell<bool>,
}

impl<'a> Dispatcher<'a> {
    pub fn new(format: InterfaceFormat) -> Dispatcher<'a> {
        Dispatcher {
            commbus: CommBus::default(),
            downloader: Rc::new(NavigationDataDownloader::new()),
            database: match format {
                InterfaceFormat::DFDv1 => RefCell::new(DatabaseV1::default().into()),
                InterfaceFormat::DFDv2 => RefCell::new(DatabaseV2::default().into()),
                InterfaceFormat::Custom => RefCell::new(DatabaseManual::default().into()),
            },
            delta_time: std::time::Duration::from_secs(u64::MAX), /* Initialize to max so that we send a heartbeat on
                                                                   * the first update */
            queue: Rc::new(RefCell::new(Vec::new())),
            set_active_on_finish: RefCell::new(false),
        }
    }

    fn list_packages(&self, sort: bool, filter: bool) -> Vec<PackageInfo> {
        let navigation_data_path = Path::new(consts::NAVIGATION_DATA_WORK_LOCATION);

        if !Path::exists(navigation_data_path) {
            fs::create_dir(navigation_data_path).unwrap();
        }

        let navigation_data_folder = fs::read_dir(navigation_data_path);

        let mut packages = vec![];

        for file in navigation_data_folder.unwrap() {
            let Ok(file) = file else {
                continue;
            };

            let file_path = file.path();

            let cycle_file = fs::File::open(file_path.join("cycle.json"));

            match cycle_file {
                Ok(cycle_file) => {
                    let cycle: InstalledNavigationDataCycleInfo = serde_json::from_reader(cycle_file).unwrap();

                    let uuid = match file_path.file_name().unwrap().to_string_lossy().to_string().as_str() {
                        "active" => generate_uuid_from_cycle(&cycle),
                        x => String::from(x),
                    };

                    packages.push(PackageInfo {
                        path: String::from(file_path.to_string_lossy()),
                        uuid,
                        cycle,
                    });
                },
                Err(err) => eprintln!("{:?}", err),
            }
        }

        if filter {
            let interface_type = self.database.borrow().get_database_type().as_str().to_string();

            packages.retain(|package| *package.cycle.format == interface_type);
        }

        if sort {
            packages.sort_by(|a, b| {
                b.cycle
                    .cycle
                    .cmp(&a.cycle.cycle)
                    .then(b.cycle.revision.cmp(&a.cycle.revision))
                    .then(a.cycle.format.cmp(&b.cycle.format))
            });
        }

        packages
    }

    fn set_package(&self, uuid: String) -> Result<bool, Box<dyn Error>> {
        let base_path = Path::new(consts::NAVIGATION_DATA_WORK_LOCATION);

        let active_path = base_path.join("active");

        let uuid_path = base_path.join(uuid.clone());

        if path_exists(&active_path) {
            let file_handle = fs::File::open(active_path.join("cycle.json")).unwrap();

            let cycle: InstalledNavigationDataCycleInfo = serde_json::from_reader(file_handle).unwrap();

            let hash = generate_uuid_from_cycle(&cycle);

            if hash == uuid {
                return Ok(false);
            }

            let package: PackageInfo = PackageInfo {
                path: String::from(active_path.to_string_lossy()),
                uuid: uuid.clone(),
                cycle,
            };

            self.database.borrow_mut().disable_cycle(package)?;

            if path_exists(&Path::new(consts::NAVIGATION_TEST_LOCATION)) {
                util::delete_folder_recursively(&active_path, None)?;
            } else {
                // Disables the old path
                match fs::rename(&active_path, base_path.join(hash)) {
                    Ok(_) => (),
                    Err(err) => eprintln!("{}", err),
                }
            }
        }

        if !path_exists(&uuid_path) {
            return Err("Package does not exist".into());
        }

        let cycle: InstalledNavigationDataCycleInfo =
            serde_json::from_reader(fs::File::open(uuid_path.join("cycle.json")).unwrap()).unwrap();

        // Check for format change and updates the used interface
        if cycle.format != self.database.borrow().get_database_type().as_str() {
            let new_format = InterfaceFormat::from(&cycle.format);

            self.database.replace(match new_format {
                InterfaceFormat::DFDv1 => DatabaseV1::default().into(),
                InterfaceFormat::DFDv2 => DatabaseV2::default().into(),
                InterfaceFormat::Custom => DatabaseManual::default().into(),
            });
        }

        let package: PackageInfo = PackageInfo {
            path: String::from(active_path.to_string_lossy()),
            uuid,
            cycle,
        };

        fs::rename(uuid_path.clone(), active_path)?;

        let db_set = self.database.borrow_mut().enable_cycle(package)?;

        if db_set {
            println!("[NAVIGRAPH]: Set Successful");
        } else {
            println!("[NAVIGRAPH]: Set Unsuccessful");
        };

        Ok(db_set)
    }

    fn setup_packages(&self) -> Result<String, Box<dyn Error>> {
        self.copy_bundles()?;

        // Auto enable already activated cycle
        let work_path = Path::new(consts::NAVIGATION_DATA_WORK_LOCATION);
        let active_path = work_path.join("active");

        if path_exists(&Path::new(consts::NAVIGATION_TEST_LOCATION)) {
            // Testing shim
            return Ok(String::from("Test Initalized"));
        } else if path_exists(&active_path) {
            let cycle: InstalledNavigationDataCycleInfo =
                serde_json::from_reader(fs::File::open(active_path.join("cycle.json")).unwrap()).unwrap();

            if cycle.format != self.database.borrow().get_database_type().as_str() {
                let new_format = InterfaceFormat::from(&cycle.format);

                self.database.replace(match new_format {
                    InterfaceFormat::DFDv1 => DatabaseV1::default().into(),
                    InterfaceFormat::DFDv2 => DatabaseV2::default().into(),
                    InterfaceFormat::Custom => DatabaseManual::default().into(),
                });
            }

            let hash = generate_uuid_from_cycle(&cycle);

            let package: PackageInfo = PackageInfo {
                path: String::from(active_path.to_string_lossy()),
                uuid: hash,
                cycle,
            };

            self.database.borrow_mut().enable_cycle(package)?;
        } else {
            let packages = self.list_packages(true, false);

            if packages.is_empty() {
                return Err("No packages found to initialize".into());
            }

            self.set_package(packages[0].uuid.clone())?;
        }

        Ok(String::from("Packages Setup"))
    }

    fn copy_bundles(&self) -> Result<bool, Box<dyn Error>> {
        let bundled_path = Path::new(consts::NAVIGATION_DATA_DEFAULT_LOCATION);

        let package_list = self.list_packages(false, false);

        let uuid_list: Vec<String> = package_list.iter().map(|package| package.uuid.clone()).collect();

        let mut active_uuid: String = String::new();

        let active_path = Path::new(consts::NAVIGATION_DATA_WORK_LOCATION).join("active");

        if path_exists(&active_path) {
            active_uuid = generate_uuid_from_path(active_path.join("cycle.json"))?;
        }

        let Ok(bundled_dir) = fs::read_dir(bundled_path) else {
            println!("[NAVIGRAPH]: No Bundled Data");
            return Ok(false);
        };

        for file in bundled_dir {
            let Ok(file) = file else {
                continue;
            };

            let cycle_path = file.path().join("cycle.json");

            if !Path::exists(&cycle_path) {
                println!(
                    "[NAVIGRAPH]: Can't find cycle.json in {}",
                    file.path().to_string_lossy()
                );
                continue;
            }

            let cycle_hypenated = generate_uuid_from_path(cycle_path)?;

            // This should work, however it does not
            if uuid_list.contains(&cycle_hypenated) {
                continue;
            }

            // This shall exist until I fix the copying bug, crashes sim, hard to debug manually.
            if cycle_hypenated == active_uuid {
                continue;
            }

            let work_path = Path::new(consts::NAVIGATION_DATA_WORK_LOCATION).join(cycle_hypenated);

            util::copy_files_to_folder(&file.path(), &work_path)?;
        }

        Ok(true)
    }

    fn delete_package(&self, uuid: String) -> io::Result<()> {
        let package_path = Path::new(consts::NAVIGATION_DATA_WORK_LOCATION).join(uuid);

        util::delete_folder_recursively(&package_path, None)
    }

    fn clean_up_packages(&self, count_max: Option<i32>) -> Result<(), Box<dyn Error>> {
        let bundle_path = Path::new(consts::NAVIGATION_DATA_DEFAULT_LOCATION);

        let mut bundle_ids = vec![];

        for dir in bundle_path.read_dir()? {
            let Ok(dir) = dir else {
                continue;
            };

            bundle_ids.push(generate_uuid_from_path(dir.path().join("cycle.json"))?);
        }

        let packages = self.list_packages(true, false);

        let mut count = 0;

        let (_keep, delete): (Vec<PackageInfo>, Vec<PackageInfo>) = packages.into_iter().partition(|pkg| {
            if (self.database.borrow().get_database_type().as_str() == pkg.cycle.format)
                && (count <= count_max.unwrap_or(3))
            {
                count += 1;
                return true;
            } else if bundle_ids.contains(&pkg.uuid) || pkg.path.contains("active") {
                return true;
            }
            false
        });

        for pkg in delete {
            self.delete_package(pkg.uuid)?;
        }

        Ok(())
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
        // Runs before everything, used to set up the navdata in the right places.
        match self.setup_packages() {
            Ok(_) => (),
            Err(x) => eprintln!("Packages failed to setup, Err: {}", x),
        }

        // Runs extra setup on the configured database format handler
        self.database.borrow().setup().unwrap();

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

        // Because the download process doesn't finish in the function call, we need to check if the download is
        // finished to call the on_download_finish function
        let download_status = self.downloader.download_status.borrow().clone();

        if let DownloadStatus::Downloaded(package_uuid) = download_status {
            self.on_download_finish(package_uuid);
            self.downloader.acknowledge_download();
        }
    }

    // TODO: Implement possible db switching on finish
    fn on_download_finish(&mut self, package_uuid: String) {
        if *self.set_active_on_finish.borrow() {
            self.set_package(package_uuid).unwrap_or_default();
            self.set_active_on_finish.replace(false);
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

                    // Get params for the set active when the download is finished
                    let params = task
                        .borrow()
                        .parse_data_as::<params::DownloadNavigationDataParams>()
                        .unwrap();

                    self.set_active_on_finish.replace(params.set_active.unwrap_or(false));

                    self.downloader.download(Rc::clone(task));
                },
                FunctionType::SetDownloadOptions => {
                    Dispatcher::execute_task(task.clone(), |t| self.downloader.set_download_options(t))
                },
                FunctionType::GetNavigationDataInstallStatus => {
                    // We can't use the execute_task function here because the download process doesn't finish in the
                    // function call, which results in slightly "messier" code

                    // We first need to initialize the network request and then wait for the response
                    meta::start_network_request(Rc::clone(task))
                },
                FunctionType::ListAvailablePackages => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::ListAvailablePackages>()?;

                    let packages = self.list_packages(params.sort.unwrap_or(false), params.filter.unwrap_or(false));

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(packages)?));

                    Ok(())
                }),
                FunctionType::SetActivePackage => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::SetActivePackage>()?;
                    let data = self.set_package(params.uuid)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(data)?));

                    Ok(())
                }),
                FunctionType::DeletePackage => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::DeletePackage>()?;
                    self.delete_package(params.uuid)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(().into()));

                    Ok(())
                }),
                FunctionType::CleanPackages => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::CleanPackages>()?;
                    self.clean_up_packages(params.count)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(().into()));

                    Ok(())
                }),
                FunctionType::ExecuteSQLQuery => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::ExecuteSQLQueryParams>()?;
                    let data = self.database.borrow().execute_sql_query(params.sql, params.params)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(data));

                    Ok(())
                }),
                FunctionType::GetDatabaseInfo => Dispatcher::execute_task(task.clone(), |t| {
                    let info = self.database.borrow().get_database_info()?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(info)?));

                    Ok(())
                }),
                FunctionType::GetAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let airport = self.database.borrow().get_airport(params.ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(airport)?));

                    Ok(())
                }),
                FunctionType::GetWaypoints => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let waypoints = self.database.borrow().get_waypoints(params.ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(waypoints)?));

                    Ok(())
                }),
                FunctionType::GetVhfNavaids => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let vhf_navaids = self.database.borrow().get_vhf_navaids(params.ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(vhf_navaids)?));

                    Ok(())
                }),
                FunctionType::GetNdbNavaids => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let ndb_navaids = self.database.borrow().get_ndb_navaids(params.ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(ndb_navaids)?));

                    Ok(())
                }),
                FunctionType::GetAirways => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetByIdentParas>()?;
                    let airways = self.database.borrow().get_airways(params.ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(airways)?));

                    Ok(())
                }),
                FunctionType::GetAirwaysAtFix => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtFixParams>()?;
                    let airways = self
                        .database
                        .borrow()
                        .get_airways_at_fix(params.fix_ident, params.fix_icao_code)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(airways)?));

                    Ok(())
                }),
                FunctionType::GetAirportsInRange => Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let airports = self
                        .database
                        .borrow()
                        .get_airports_in_range(params.center, params.range)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(airports)?));

                    Ok(())
                }),
                FunctionType::GetWaypointsInRange => Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let waypoints = self
                        .database
                        .borrow()
                        .get_waypoints_in_range(params.center, params.range)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(waypoints)?));

                    Ok(())
                }),
                FunctionType::GetVhfNavaidsInRange => Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let navaids = self
                        .database
                        .borrow()
                        .get_vhf_navaids_in_range(params.center, params.range)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(navaids)?));

                    Ok(())
                }),
                FunctionType::GetNdbNavaidsInRange => Dispatcher::execute_task(task.clone(), |t: Rc<RefCell<Task>>| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let navaids = self
                        .database
                        .borrow()
                        .get_ndb_navaids_in_range(params.center, params.range)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(navaids)?));

                    Ok(())
                }),
                FunctionType::GetAirwaysInRange => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let airways = self
                        .database
                        .borrow()
                        .get_airways_in_range(params.center, params.range)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(airways)?));

                    Ok(())
                }),
                FunctionType::GetControlledAirspacesInRange => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let airspaces = self
                        .database
                        .borrow()
                        .get_controlled_airspaces_in_range(params.center, params.range)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(airspaces)?));

                    Ok(())
                }),
                FunctionType::GetRestrictiveAirspacesInRange => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let airspaces = self
                        .database
                        .borrow()
                        .get_restrictive_airspaces_in_range(params.center, params.range)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(airspaces)?));

                    Ok(())
                }),
                FunctionType::GetCommunicationsInRange => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetInRangeParams>()?;
                    let communications = self
                        .database
                        .borrow()
                        .get_communications_in_range(params.center, params.range)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(communications)?));

                    Ok(())
                }),
                FunctionType::GetRunwaysAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let runways = self.database.borrow().get_runways_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(runways)?));

                    Ok(())
                }),
                FunctionType::GetDeparturesAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let departures = self.database.borrow().get_departures_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(departures)?));

                    Ok(())
                }),
                FunctionType::GetArrivalsAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let arrivals = self.database.borrow().get_arrivals_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(arrivals)?));

                    Ok(())
                }),
                FunctionType::GetApproachesAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let arrivals = self.database.borrow().get_approaches_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(arrivals)?));

                    Ok(())
                }),
                FunctionType::GetWaypointsAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let waypoints = self.database.borrow().get_waypoints_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(waypoints)?));

                    Ok(())
                }),
                FunctionType::GetNdbNavaidsAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let navaids = self
                        .database
                        .borrow()
                        .get_ndb_navaids_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(navaids)?));

                    Ok(())
                }),
                FunctionType::GetGatesAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let gates = self.database.borrow().get_gates_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(gates)?));

                    Ok(())
                }),
                FunctionType::GetCommunicationsAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let communications = self
                        .database
                        .borrow()
                        .get_communications_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(communications)?));

                    Ok(())
                }),
                FunctionType::GetGlsNavaidsAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let navaids = self
                        .database
                        .borrow()
                        .get_gls_navaids_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(navaids)?));

                    Ok(())
                }),
                FunctionType::GetPathPointsAtAirport => Dispatcher::execute_task(task.clone(), |t| {
                    let params = t.borrow().parse_data_as::<params::GetAtAirportParams>()?;
                    let pathpoints = self
                        .database
                        .borrow()
                        .get_path_points_at_airport(params.airport_ident)?;

                    t.borrow_mut().status = TaskStatus::Success(Some(serde_json::to_value(pathpoints)?));

                    Ok(())
                }),
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
                    },
                    _ => {
                        // Should not happen for now
                        println!("[NAVIGRAPH] Network request completed but no handler for this type of request");
                    },
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
                },
                TaskStatus::Failure(ref error) => {
                    status = FunctionStatus::Error;
                    data = Some(error.clone().into());
                    (status, data)
                },
                _ => unreachable!(), // This should never happen
            };

            let json = FunctionResult {
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
                println!("[NAVIGRAPH] Task failed: {}", e);
                task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            },
        }
    }

    fn add_to_queue(queue: Rc<RefCell<Vec<Rc<RefCell<Task>>>>>, args: &str) -> Result<(), Box<dyn std::error::Error>> {
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
            CommBus::call("NAVIGRAPH_Event", &serialized_json, CommBusBroadcastFlags::All);
        } else {
            println!("[NAVIGRAPH] Failed to serialize event");
        }
    }
}
