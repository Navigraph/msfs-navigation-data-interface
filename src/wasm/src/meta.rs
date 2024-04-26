use std::{
    cell::RefCell,
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

#[derive(serde::Serialize, Debug)]
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

#[derive(serde::Deserialize)]
pub struct CurrentCycleResponse {
    pub name: String,
    pub version: String,
    pub configuration: String,
    pub cycle: String,
}

#[derive(serde::Deserialize)]
pub struct InstalledNavigationDataCycleInfo {
    pub cycle: String,
    pub revision: String,
    pub name: String,
    pub format: String,
    #[serde(rename = "validityPeriod")]
    pub validity_period: String,
}

pub fn start_network_request(task: Rc<RefCell<Task>>) {
    let request = NetworkHelper::make_request("https://navdata.api.navigraph.com/info", Method::Get, None, None);
    let request = match request {
        Ok(request) => request,
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        },
    };
    task.borrow_mut().associated_network_request = Some(request);
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
                    },
                }
            } else {
                return;
            }
        },
        None => {
            task.borrow_mut().status = TaskStatus::Failure("No associated network request".to_string());
            return;
        },
    };

    let response_struct: CurrentCycleResponse = match serde_json::from_slice(&response_bytes) {
        Ok(response_struct) => response_struct,
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        },
    };

    // figure out install status
    let found_downloaded = path_exists(Path::new(consts::NAVIGATION_DATA_DOWNLOADED_LOCATION));

    let found_bundled = path_exists(Path::new(consts::NAVIGATION_DATA_DEFAULT_LOCATION));

    let status = if found_downloaded {
        InstallStatus::Manual
    } else if found_bundled {
        InstallStatus::Bundled
    } else {
        InstallStatus::None
    };

    // Open JSON
    let json_path = match status {
        InstallStatus::Manual => Some(PathBuf::from(consts::NAVIGATION_DATA_DOWNLOADED_LOCATION).join("cycle.json")),
        InstallStatus::Bundled => Some(PathBuf::from(consts::NAVIGATION_DATA_DEFAULT_LOCATION).join("cycle.json")),
        InstallStatus::None => None,
    };

    let installed_cycle_info = match json_path {
        Some(json_path) => {
            let json_file = match std::fs::File::open(json_path) {
                Ok(json_file) => json_file,
                Err(e) => {
                    task.borrow_mut().status = TaskStatus::Failure(e.to_string());
                    return;
                },
            };

            let installed_cycle_info: InstalledNavigationDataCycleInfo = match serde_json::from_reader(json_file) {
                Ok(installed_cycle_info) => installed_cycle_info,
                Err(e) => {
                    task.borrow_mut().status = TaskStatus::Failure(e.to_string());
                    return;
                },
            };

            Some(installed_cycle_info)
        },
        None => None,
    };

    let status = NavigationDataStatus {
        status,
        installed_format: match &installed_cycle_info {
            Some(installed_cycle_info) => Some(installed_cycle_info.format.clone()),
            None => None,
        },
        installed_revision: match &installed_cycle_info {
            Some(installed_cycle_info) => Some(installed_cycle_info.revision.clone()),
            None => None,
        },
        installed_cycle: match &installed_cycle_info {
            Some(installed_cycle_info) => Some(installed_cycle_info.cycle.clone()),
            None => None,
        },
        install_path: match status {
            InstallStatus::Manual => Some(consts::NAVIGATION_DATA_DOWNLOADED_LOCATION.to_string()),
            InstallStatus::Bundled => Some(consts::NAVIGATION_DATA_DEFAULT_LOCATION.to_string()),
            InstallStatus::None => None,
        },
        validity_period: match &installed_cycle_info {
            Some(installed_cycle_info) => Some(installed_cycle_info.validity_period.clone()),
            None => None,
        },
        latest_cycle: response_struct.cycle,
    };

    let status_as_value = match serde_json::to_value(&status) {
        Ok(status_as_value) => status_as_value,
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        },
    };

    println!("Status: {:#?}", status);

    task.borrow_mut().status = TaskStatus::Success(Some(status_as_value));
}
