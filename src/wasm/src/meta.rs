use std::{
    cell::RefCell,
    error::Error,
    path::{Path, PathBuf},
    rc::Rc,
};

use msfs::network::NetworkRequestState;

use crate::{
    consts,
    dispatcher::{Task, TaskStatus},
    network_helper::{Method, NetworkHelper},
    util::path_exists,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct InternalState {
    pub is_bundled: bool,
}

#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallStatus {
    Bundled,
    Manual,
    None,
}

#[derive(serde::Serialize, Debug)]
pub struct NavigationDataStatus {
    pub status: InstallStatus,
    #[serde(rename = "installedFormat")]
    pub installed_format: Option<String>,
    #[serde(rename = "installedRevision")]
    pub installed_revision: Option<String>,
    #[serde(rename = "installedCycle")]
    pub installed_cycle: Option<String>,
    #[serde(rename = "installedPath")]
    pub install_path: Option<String>,
    #[serde(rename = "validityPeriod")]
    pub validity_period: Option<String>,
    #[serde(rename = "latestCycle")]
    pub latest_cycle: String,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct CurrentCycleResponse {
    pub name: String,
    pub version: String,
    pub configuration: String,
    pub cycle: String,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct InstalledNavigationDataCycleInfo {
    pub cycle: String,
    pub revision: String,
    pub name: String,
    pub format: String,
    #[serde(rename = "validityPeriod")]
    pub validity_period: String,
}

pub fn get_internal_state() -> Result<InternalState, Box<dyn Error>> {
    let config_path = Path::new(consts::NAVIGATION_DATA_INTERNAL_CONFIG_LOCATION);
    if !path_exists(config_path) {
        Err("Internal config file does not exist")?;
    }

    let config_file = std::fs::File::open(config_path)?;
    let internal_state: InternalState = serde_json::from_reader(config_file)?;

    Ok(internal_state)
}

pub fn set_internal_state(internal_state: InternalState) -> Result<(), Box<dyn Error>> {
    let config_path = Path::new(consts::NAVIGATION_DATA_INTERNAL_CONFIG_LOCATION);
    let config_file = std::fs::File::create(config_path)?;
    serde_json::to_writer(config_file, &internal_state)?;

    Ok(())
}

pub fn start_network_request(task: Rc<RefCell<Task>>) {
    let request = NetworkHelper::make_request(
        "https://navdata.api.navigraph.com/info",
        Method::Get,
        None,
        None,
    );
    let request = match request {
        Ok(request) => request,
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        }
    };
    task.borrow_mut().associated_network_request = Some(request);
}

pub fn get_installed_cycle_from_json(
    path: &Path,
) -> Result<InstalledNavigationDataCycleInfo, Box<dyn Error>> {
    let json_file = std::fs::File::open(path)?;
    let installed_cycle_info: InstalledNavigationDataCycleInfo =
        serde_json::from_reader(json_file)?;

    Ok(installed_cycle_info)
}

pub fn get_navigation_data_install_status(task: Rc<RefCell<Task>>) {
    let response_bytes = match task.borrow().associated_network_request.as_ref() {
        Some(request) => {
            if request.response_state() == NetworkRequestState::DataReady {
                let response = request.get_response();
                match response {
                    Ok(response) => response,
                    Err(e) => {
                        task.borrow_mut().status = TaskStatus::Failure(e.to_string());
                        return;
                    }
                }
            } else {
                return;
            }
        }
        None => {
            task.borrow_mut().status =
                TaskStatus::Failure("No associated network request".to_string());
            return;
        }
    };

    let response_struct: CurrentCycleResponse = match serde_json::from_slice(&response_bytes) {
        Ok(response_struct) => response_struct,
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        }
    };

    // figure out install status
    let found_downloaded = path_exists(Path::new(consts::NAVIGATION_DATA_WORK_LOCATION));

    let found_bundled = get_internal_state()
        .map(|internal_state| internal_state.is_bundled)
        .unwrap_or(false);

    // Check bundled first, as downloaded and bundled are both possible
    let status = if found_bundled {
        InstallStatus::Bundled
    } else if found_downloaded {
        InstallStatus::Manual
    } else {
        InstallStatus::None
    };

    // Open JSON
    let json_path = if status != InstallStatus::None {
        Some(PathBuf::from(consts::NAVIGATION_DATA_WORK_LOCATION).join("cycle.json"))
    } else {
        None
    };

    let installed_cycle_info = match json_path {
        Some(json_path) => {
            let json_file = match std::fs::File::open(json_path) {
                Ok(json_file) => json_file,
                Err(e) => {
                    task.borrow_mut().status = TaskStatus::Failure(e.to_string());
                    return;
                }
            };

            let installed_cycle_info: InstalledNavigationDataCycleInfo =
                match serde_json::from_reader(json_file) {
                    Ok(installed_cycle_info) => installed_cycle_info,
                    Err(e) => {
                        task.borrow_mut().status = TaskStatus::Failure(e.to_string());
                        return;
                    }
                };

            Some(installed_cycle_info)
        }
        None => None,
    };

    let navigation_data_status = NavigationDataStatus {
        status,
        installed_format: installed_cycle_info.as_ref().map(|installed_cycle_info| installed_cycle_info.format.clone()),
        installed_revision: installed_cycle_info.as_ref().map(|installed_cycle_info| installed_cycle_info.revision.clone()),
        installed_cycle: installed_cycle_info.as_ref().map(|installed_cycle_info| installed_cycle_info.cycle.clone()),
        install_path: if status == InstallStatus::Manual {
            Some(consts::NAVIGATION_DATA_WORK_LOCATION.to_string())
        } else {
            None
        },
        validity_period: installed_cycle_info.as_ref().map(|installed_cycle_info| installed_cycle_info.validity_period.clone()),
        latest_cycle: response_struct.cycle,
    };

    let status_as_value = match serde_json::to_value(navigation_data_status) {
        Ok(status_as_value) => status_as_value,
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        }
    };

    task.borrow_mut().status = TaskStatus::Success(Some(status_as_value));
}
