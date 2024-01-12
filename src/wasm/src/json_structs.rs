//! Contains structs relating to JSON data

/// Contains structs relating to functions
pub mod functions {
    #[derive(serde::Deserialize, Clone, Copy)]
    pub enum FunctionType {
        #[serde(rename = "DownloadNavdata")]
        DownloadNavdata,
        #[serde(rename = "SetDownloadOptions")]
        SetDownloadOptions,
        #[serde(rename = "SetActiveDatabase")]
        SetActiveDatabase,
        #[serde(rename = "ExecuteSQLQuery")]
        ExecuteSQLQuery,
        #[serde(rename = "GetAirport")]
        GetAirport,
        #[serde(rename = "GetAirportsInRange")]
        GetAirportsInRange,
        #[serde(rename = "GetAirways")]
        GetAirways,
        #[serde(rename = "GetAirwaysInRange")]
        GetAirwaysInRange,
        #[serde(rename = "GetDeparturesAtAirport")]
        GetDeparturesAtAirport,
        #[serde(rename = "GetArrivalsAtAirport")]
        GetArrivalsAtAirport,
    }

    #[derive(serde::Serialize)]
    pub enum FunctionStatus {
        #[serde(rename = "Error")]
        Error,
        #[serde(rename = "Success")]
        Success,
    }

    #[derive(serde::Deserialize)]
    pub struct CallFunction {
        /// Type of function to call
        pub function: FunctionType,
        /// The unique ID of the function call
        pub id: String,
        /// Data associated with the function call
        pub data: Option<serde_json::Value>,
    }

    #[derive(serde::Serialize)]
    pub struct FunctionResult {
        /// The unique ID of the function call
        pub id: String,
        /// Status of the function call
        pub status: FunctionStatus,
        /// Data associated with the function call
        pub data: Option<serde_json::Value>,
    }
}

/// Contains structs relating to events
pub mod events {

    #[derive(serde::Serialize)]
    pub enum EventType {
        #[serde(rename = "Heartbeat")]
        Heartbeat,
        #[serde(rename = "DownloadProgress")]
        DownloadProgress,
    }

    #[derive(serde::Serialize)]
    pub struct Event {
        /// Type of event
        pub event: EventType,
        /// Data associated with the event
        pub data: Option<serde_json::Value>,
    }

    #[derive(serde::Serialize)]
    pub enum DownloadProgressPhase {
        #[serde(rename = "Downloading")]
        Downloading,
        #[serde(rename = "Cleaning")]
        Cleaning,
        #[serde(rename = "Extracting")]
        Extracting,
    }

    #[derive(serde::Serialize)]
    pub struct DownloadProgressEvent {
        /// Phase of the download
        pub phase: DownloadProgressPhase,
        /// Number of files deleted so far
        pub deleted: Option<usize>,
        /// Total number of files to unzip
        pub total_to_unzip: Option<usize>,
        /// Number of files unzipped so far
        pub unzipped: Option<usize>,
    }
}

/// Contains structs relating to parameters
pub mod params {
    use navigation_database::math::{Coordinates, NauticalMiles};

    #[derive(serde::Deserialize)]
    pub struct DownloadNavdataParams {
        /// Path to the folder to download to
        pub path: String,
        /// URL to download from
        pub url: String,
    }

    #[derive(serde::Deserialize)]
    pub struct SetDownloadOptionsParams {
        /// Batch size for deleting/extracting files
        pub batch_size: usize,
    }

    #[derive(serde::Deserialize)]
    pub struct SetActiveDatabaseParams {
        /// Path to the DFD database file
        pub path: String,
    }

    #[derive(serde::Deserialize)]
    pub struct ExecuteSQLQueryParams {
        /// SQL query to execute
        pub sql: String,
        pub params: Vec<String>,
    }

    #[derive(serde::Deserialize)]
    pub struct GetAirportParams {
        /// identifier of the airport
        pub ident: String,
    }

    #[derive(serde::Deserialize)]
    pub struct GetAirportsInRangeParams {
        pub center: Coordinates,
        pub range: NauticalMiles,
    }

    #[derive(serde::Deserialize)]
    pub struct GetAirwaysParams {
        pub ident: String,
    }

    #[derive(serde::Deserialize)]
    pub struct GetAirwaysInRangeParams {
        pub center: Coordinates,
        pub range: NauticalMiles,
    }

    #[derive(serde::Deserialize)]
    pub struct GetProceduresAtAirportParams {
        pub airport_ident: String,
    }
}
