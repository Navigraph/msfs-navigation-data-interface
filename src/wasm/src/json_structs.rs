//! Contains structs relating to JSON data

/// Contains structs relating to functions
pub mod functions {
    #[derive(serde::Deserialize, Clone, Copy, PartialEq)]
    pub enum FunctionType {
        /// `DownloadNavigationDataParams`
        DownloadNavigationData,
        /// `SetDownloadOptionsParams`
        SetDownloadOptions,
        /// `ExecuteSQLQueryParams`
        ExecuteSQLQuery,

        /// no Params
        GetDatabaseInfo,
        /// `ListAvailablePackages`
        ListAvailablePackages,
        /// `SetActivePackage`
        SetActivePackage,
        /// `GetActivePackage`
        GetActivePackage,
        /// `DeletePackage`
        DeletePackage,
        /// `CleanPackages`
        CleanPackages,

        /// `GetByIdentParams`
        GetAirport,
        /// `GetByIdentParams`
        GetWaypoints,
        /// `GetByIdentParams`
        GetVhfNavaids,
        /// `GetByIdentParams`
        GetNdbNavaids,
        /// `GetByIdentParams`
        GetAirways,
        /// `GetByIdentParams`
        GetAirwaysAtFix,

        /// `GetInRangeParams`
        GetAirportsInRange,
        /// `GetInRangeParams`
        GetWaypointsInRange,
        /// `GetInRangeParams`
        GetVhfNavaidsInRange,
        /// `GetInRangeParams`
        GetNdbNavaidsInRange,
        /// `GetInRangeParams`
        GetAirwaysInRange,
        /// `GetInRangeParams`
        GetControlledAirspacesInRange,
        /// `GetInRangeParams`
        GetRestrictiveAirspacesInRange,
        /// `GetInRangeParams`
        GetCommunicationsInRange,

        /// `GetAtAirportParams`
        GetRunwaysAtAirport,
        /// `GetAtAirportParams`
        GetDeparturesAtAirport,
        /// `GetAtAirportParams`
        GetArrivalsAtAirport,
        /// `GetAtAirportParams`
        GetApproachesAtAirport,
        /// `GetAtAirportParams`
        GetWaypointsAtAirport,
        /// `GetAtAirportParams`
        GetNdbNavaidsAtAirport,
        /// `GetAtAirportParams`
        GetGatesAtAirport,
        /// `GetAtAirportParams`
        GetCommunicationsAtAirport,
        /// `GetAtAirportParams`
        GetGlsNavaidsAtAirport,
        /// `GetAtAirportParams`
        GetPathPointsAtAirport,
    }

    #[derive(serde::Serialize)]
    pub enum FunctionStatus {
        Error,
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
        Heartbeat,
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
        Downloading,
        Cleaning,
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
    #[serde(rename_all = "camelCase")]
    pub struct DownloadNavigationDataParams {
        /// URL to download from
        pub url: String,
        pub set_active: Option<bool>,
    }

    #[derive(serde::Deserialize)]
    pub struct SetDownloadOptionsParams {
        /// Batch size for deleting/extracting files
        pub batch_size: usize,
    }

    #[derive(serde::Deserialize)]
    pub struct ListAvailablePackages {
        pub sort: Option<bool>,
        pub filter: Option<bool>,
    }

    #[derive(serde::Deserialize)]
    pub struct SetActivePackage {
        /// UUID that the package is stored as
        pub uuid: String,
    }

    #[derive(serde::Deserialize)]
    pub struct DeletePackage {
        pub uuid: String,
    }

    #[derive(serde::Deserialize)]
    pub struct CleanPackages {
        pub count: Option<i32>,
    }

    #[derive(serde::Deserialize)]
    pub struct ExecuteSQLQueryParams {
        /// SQL query to execute
        pub sql: String,
        pub params: Vec<String>,
    }

    #[derive(serde::Deserialize)]
    pub struct GetByIdentParas {
        /// identifier of the item
        pub ident: String,
    }

    #[derive(serde::Deserialize)]
    pub struct GetAtFixParams {
        /// identifier of the fix
        pub fix_ident: String,
        /// icao_code of the fix
        pub fix_icao_code: String,
    }

    #[derive(serde::Deserialize)]
    pub struct GetInRangeParams {
        pub center: Coordinates,
        pub range: NauticalMiles,
    }

    #[derive(serde::Deserialize)]
    pub struct GetAtAirportParams {
        pub airport_ident: String,
    }
}
