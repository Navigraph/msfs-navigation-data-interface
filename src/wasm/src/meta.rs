use std::{cell::RefCell, path::Path, rc::Rc};

use msfs::network::NetworkRequestState;
use navigation_database::traits::InstalledNavigationDataCycleInfo;

use crate::{
    consts,
    dispatcher::{Task, TaskStatus},
    network_helper::{Method, NetworkHelper},
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

// Now just gets the status of the active database. Can still determine if you installed extra pacakges.
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

    // To find downloaded we can just take the length of the prebundled vs the installed.
    let prebundled_folder = match std::fs::read_dir(Path::new(consts::NAVIGATION_DATA_DEFAULT_LOCATION)) {
        Ok(dir) => dir.count(),
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        },
    };

    let current_count = match std::fs::read_dir(Path::new(consts::NAVIGATION_DATA_WORK_LOCATION)) {
        Ok(dir) => dir.count(),
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        },
    };

    let status = if current_count == 0 {
        InstallStatus::None
    } else if prebundled_folder >= current_count {
        InstallStatus::Bundled
    } else {
        InstallStatus::Manual
    };

    let active_path = Path::new(consts::NAVIGATION_DATA_WORK_LOCATION).join("active");

    let json_path = match status {
        InstallStatus::None => None,
        _ => Some(active_path.join("cycle.json")),
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

    let navigation_data_status = NavigationDataStatus {
        status,
        installed_format: installed_cycle_info
            .as_ref()
            .map(|installed_cycle_info| installed_cycle_info.format.clone()),
        installed_revision: installed_cycle_info
            .as_ref()
            .map(|installed_cycle_info| installed_cycle_info.revision.clone()),
        installed_cycle: installed_cycle_info
            .as_ref()
            .map(|installed_cycle_info| installed_cycle_info.cycle.clone()),
        install_path: match status {
            InstallStatus::None => None,
            _ => Some(active_path.to_string_lossy().to_string()),
        },
        validity_period: installed_cycle_info
            .as_ref()
            .map(|installed_cycle_info| installed_cycle_info.validity_period.clone()),
        latest_cycle: response_struct.cycle,
    };

    let status_as_value = match serde_json::to_value(navigation_data_status) {
        Ok(status_as_value) => status_as_value,
        Err(e) => {
            task.borrow_mut().status = TaskStatus::Failure(e.to_string());
            return;
        },
    };

    task.borrow_mut().status = TaskStatus::Success(Some(status_as_value));
}
