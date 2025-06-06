use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, Write},
};

use anyhow::{anyhow, Context, Result};
use msfs::network::NetworkRequestBuilder;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use zip::ZipArchive;

use crate::{
    database::{
        Airport, Airway, Approach, Arrival, Communication, ControlledAirspace, Coordinates,
        DatabaseInfo, Departure, Gate, GlsNavaid, NdbNavaid, PathPoint, RestrictiveAirspace,
        RunwayThreshold, VhfNavaid, Waypoint, DATABASE_STATE, WORK_CYCLE_JSON_PATH, WORK_DB_PATH,
        WORK_NAVIGATION_DATA_FOLDER,
    },
    futures::AsyncNetworkRequest,
    DownloadProgressEvent, InterfaceEvent,
};

/// The URL to get the latest available cycle number
const LATEST_CYCLE_ENDPOINT: &str = "https://navdata.api.navigraph.com/info";

/// The path to the temporary download file
const DOWNLOAD_TEMP_FILE_PATH: &str = "\\work/ng_download.temp";

/// The max size in bytes of each request during the download function (set to 4MB curently)
const DOWNLOAD_CHUNK_SIZE_BYTES: usize = 4 * 1024 * 1024;

/// The trait definition for a function that can be called through the navigation data interface
trait Function: DeserializeOwned {
    type ReturnType: Serialize;

    /// Create a new instance of the function
    ///
    /// * `data` - A `serde_json::Value` with the data passed in the call
    fn new(mut data: serde_json::Value) -> Result<Self> {
        // If data is just `null`, remap to an empty object. There are cases where devs pass null instead of {}, which causes an error here
        if data == Value::Null {
            data = json!({});
        }

        let mut instance =
            serde_json::from_value::<Self>(data).context("can't deserialize self")?;

        instance.init()?;

        Ok(instance)
    }

    /// Any custom initialization logic to call before execution
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// The main function entry
    async fn run(&mut self) -> Result<Self::ReturnType>;
}

#[derive(Deserialize)]
pub struct DownloadNavigationData {
    url: String,
}

impl Function for DownloadNavigationData {
    type ReturnType = ();

    async fn run(&mut self) -> Result<Self::ReturnType> {
        self.download_to_temp().await?;

        // Only close connection if DATABASE_STATE has already been initialized - otherwise we end up unnecessarily copying the bundled data and instantly replacing it (due to initialization logic in database state)
        if Lazy::get(&DATABASE_STATE).is_some() {
            // Drop the current database. We don't do this before the download as there is a chance it will fail, and then we end up with no database open.
            DATABASE_STATE
                .try_lock()
                .map_err(|_| anyhow!("can't lock DATABASE_STATE"))?
                .close_connection()?;
        }

        self.extract_navigation_data().await?;

        // Open the connection
        DATABASE_STATE
            .try_lock()
            .map_err(|_| anyhow!("can't lock DATABASE_STATE"))?
            .open_connection()?;

        // Remove the temp file
        fs::remove_file(DOWNLOAD_TEMP_FILE_PATH)?;

        Ok(())
    }
}

impl DownloadNavigationData {
    /// Download the navigation data zip file to the temp file location
    async fn download_to_temp(&self) -> Result<()> {
        // Figure out total size of download (this request is acting like a HEAD since we don't have those in this environment. Nothing actually gets downloaded since we are constraining the range)
        let request = NetworkRequestBuilder::new(&self.url)
            .context("can't create new NetworkRequestBuilder")?
            .with_header(&format!("Range: bytes=0-0"))
            .context(".with_header() returned None")?
            .get()
            .context(".get() returned None")?;

        request.wait_for_data().await?;

        // Try parsing the content-range header
        let total_bytes = request
            .header_section("content-range")
            .context("no content-range header")?
            .trim()
            .split("/")
            .last()
            .ok_or(anyhow!("invalid content-range"))?
            .parse::<usize>()?;

        // Total amount of chunks to download.  We need to download the data in chunks of DOWNLOAD_CHUNK_SIZE_BYTES to avoid a timeout, so we need to keep track of a "working" accumulation of all responses
        let total_chunks = total_bytes.div_ceil(DOWNLOAD_CHUNK_SIZE_BYTES);

        // Store the download to a file to avoid holding in-memory
        let mut download_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(DOWNLOAD_TEMP_FILE_PATH)?;

        for i in 0..total_chunks {
            // Calculate the range for the current chunk
            let range_start = i * DOWNLOAD_CHUNK_SIZE_BYTES;
            let range_end = ((i + 1) * DOWNLOAD_CHUNK_SIZE_BYTES - 1).min(total_bytes - 1);

            // Report the current download progress
            InterfaceEvent::send_download_progress_event(DownloadProgressEvent {
                total_bytes,
                downloaded_bytes: range_start,
                current_chunk: i,
                total_chunks,
            })?;

            // Dispatch the request
            let data = NetworkRequestBuilder::new(&self.url)
                .context("can't create new NetworkRequestBuilder")?
                .with_header(&format!("Range: bytes={range_start}-{range_end}"))
                .context(".with_header() returned None")?
                .get()
                .context(".get() returned None")?
                .wait_for_data()
                .await?;

            // Write to limit how much data we hold in memory at a time (will be a max of DOWNLOAD_CHUNK_SIZE_BYTES)
            download_file.write_all(&data)?;
        }

        Ok(())
    }

    /// Extract the navigation data files from the zip file located in the temp location
    async fn extract_navigation_data(&self) -> Result<()> {
        // Load the zip archive
        let mut zip = ZipArchive::new(BufReader::new(File::open(DOWNLOAD_TEMP_FILE_PATH)?))?;

        // Ensure parent folder exists (ignore the result as it will return an error if it already exists)
        let _ = fs::create_dir_all(WORK_NAVIGATION_DATA_FOLDER);

        // Write the cycle.json file
        let mut cycle_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(WORK_CYCLE_JSON_PATH)?;

        std::io::copy(&mut zip.by_name("cycle.json")?, &mut cycle_file)?;

        // Write the db file
        let db_name = zip
            .file_names()
            .find(|f| f.to_lowercase().ends_with(".s3db"))
            .ok_or(anyhow!(
                "unable to find sqlite db in zip from url {}",
                self.url
            ))?
            .to_owned();

        let mut db_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(WORK_DB_PATH)?;

        std::io::copy(&mut zip.by_name(&db_name)?, &mut db_file)?;

        Ok(())
    }
}

/// The return type from the latest cycle endpoint
#[derive(Deserialize)]
struct CycleResponseInfo {
    cycle: String,
}

/// The return type for the `GetNavigationDataInstallStatus` function
#[derive(Serialize)]
struct NavigationDataInstallStatus {
    status: String, // TODO: Remove this field
    #[serde(rename = "installedFormat")]
    installed_format: Option<String>,
    #[serde(rename = "installedRevision")]
    installed_revision: Option<String>,
    #[serde(rename = "installedCycle")]
    installed_cycle: Option<String>,
    #[serde(rename = "installedPath")]
    installed_path: Option<String>,
    #[serde(rename = "validityPeriod")]
    validity_period: Option<String>,
    #[serde(rename = "latestCycle")]
    latest_cycle: Option<String>,
}

#[derive(Deserialize)]
pub struct GetNavigationDataInstallStatus {}

impl Function for GetNavigationDataInstallStatus {
    type ReturnType = NavigationDataInstallStatus;
    async fn run(&mut self) -> Result<Self::ReturnType> {
        // Try to get the latest available cycle from our API. Support cases in which the user may be offline by returning a None instead
        let latest_cycle = if let Ok(res) = NetworkRequestBuilder::new(LATEST_CYCLE_ENDPOINT)
            .context("can't create new NetworkRequestBuilder")?
            .get()
            .context(".get() returned None")?
            .wait_for_data()
            .await
        {
            let response_info = serde_json::from_slice::<CycleResponseInfo>(&res)?;

            Some(response_info.cycle)
        } else {
            None
        };

        match DATABASE_STATE
            .try_lock()
            .map_err(|_| anyhow!("can't lock DATABASE_STATE"))?
            .get_cycle_info()
        {
            Ok(cycle_info) => {
                Ok(NavigationDataInstallStatus {
                    status: "Manual".to_string(), // To simplify our code, we are just going to report "Manual" always. This should have no adverse affect as no third-party should be relying on the difference between this and "Bundled" (leftovers from pre-rewrite)
                    installed_format: Some(cycle_info.format),
                    installed_revision: Some(cycle_info.revision),
                    installed_cycle: Some(cycle_info.cycle),
                    installed_path: Some(WORK_DB_PATH.to_owned()),
                    validity_period: Some(cycle_info.validity_period),
                    latest_cycle,
                })
            }
            Err(_) => Ok(NavigationDataInstallStatus {
                status: "None".to_string(),
                installed_format: None,
                installed_revision: None,
                installed_cycle: None,
                installed_path: None,
                validity_period: None,
                latest_cycle,
            }),
        }
    }
}

/// A convenience macro to reduce the boilerplate function definitions for the database query functions.
///
/// # Example
/// ```rust
/// make_function!(
///     FunctionName {
///         required_param: String,
///     } => FunctionReturnType : function_on_database(required_param)
/// );
/// ```
///
/// The macro will generate an implementation of the `FunctionName` struct that implements `Function`, using `required_param` as what is parsed by serde when calling.
/// `FunctionReturnType` must implement `Serialize`. The underlying functionality will call `function_on_database` on the global `DatabaseState`.
///
/// The implementation generated will look like the following:
///
/// ```rust
/// #[derive(serde::Deserialize)]
/// pub struct FunctionName {
///     pub required_param: String
/// }
///
/// impl Function for FunctionName {
///     type ReturnType = FunctionReturnType;
///
///     async fn run(&mut self) -> Result<Self::ReturnType> {
///         let data = STATE.try_lock().map_err(|_| anyhow!("can't lock STATE"))?.function_on_database(self.required_param)?;
///         Ok(data)
///     }
/// }
/// ```
macro_rules! make_function {
    (
        $struct_name:ident {
            $( $field:ident : $field_ty:ty ),* $(,)?
        }
        => $return_ty:ty : $method:ident ( $( $arg:ident ),* )
    ) => {
        #[derive(serde::Deserialize)]
        pub struct $struct_name {
            $( pub $field: $field_ty ),*
        }

        impl Function for $struct_name {
            type ReturnType = $return_ty;

            async fn run(&mut self) -> Result<Self::ReturnType> {
                let data = DATABASE_STATE
                .try_lock()
                .map_err(|_| anyhow!("can't lock DATABASE_STATE"))?.$method($( &self.$arg ),*)?;
                Ok(data)
            }
        }
    };
}

make_function!(
    GetDatabaseInfo {} => DatabaseInfo : get_database_info()
);

make_function!(
    ExecuteSQLQuery {
        sql: String,
        params: Vec<String>
    } => serde_json::Value : execute_sql_query(sql, params)
);

make_function!(
    GetAirport {
        ident: String
    } => Airport : get_airport(ident)
);

make_function!(
    GetWaypoints {
        ident: String
    } => Vec<Waypoint> : get_waypoints(ident)
);

make_function!(
    GetVhfNavaids {
        ident: String
    } => Vec<VhfNavaid> : get_vhf_navaids(ident)
);

make_function!(
    GetNdbNavaids {
        ident: String
    } => Vec<NdbNavaid> : get_ndb_navaids(ident)
);

make_function!(
    GetAirways {
        ident: String
    } => Vec<Airway> : get_airways(ident)
);

make_function!(
    GetAirwaysAtFix {
        fix_ident: String,
        fix_icao_code: String
    } => Vec<Airway> : get_airways_at_fix(fix_ident, fix_icao_code)
);

make_function!(
    GetAirportsInRange {
        center: Coordinates,
        range: f64
    } => Vec<Airport> : get_airports_in_range(center, range)
);

make_function!(
    GetWaypointsInRange {
        center: Coordinates,
        range: f64
    } => Vec<Waypoint> : get_waypoints_in_range(center, range)
);

make_function!(
    GetVhfNavaidsInRange {
        center: Coordinates,
        range: f64
    } => Vec<VhfNavaid> : get_vhf_navaids_in_range(center, range)
);

make_function!(
    GetNdbNavaidsInRange {
        center: Coordinates,
        range: f64
    } => Vec<NdbNavaid> : get_ndb_navaids_in_range(center, range)
);

make_function!(
    GetAirwaysInRange {
        center: Coordinates,
        range: f64
    } => Vec<Airway> : get_airways_in_range(center, range)
);

make_function!(
    GetControlledAirspacesInRange {
        center: Coordinates,
        range: f64
    } => Vec<ControlledAirspace> : get_controlled_airspaces_in_range(center, range)
);

make_function!(
    GetRestrictiveAirspacesInRange {
        center: Coordinates,
        range: f64
    } => Vec<RestrictiveAirspace> : get_restrictive_airspaces_in_range(center, range)
);

make_function!(
    GetCommunicationsInRange {
        center: Coordinates,
        range: f64
    } => Vec<Communication> : get_communications_in_range(center, range)
);

make_function!(
    GetRunwaysAtAirport {
        airport_ident: String
    } => Vec<RunwayThreshold> : get_runways_at_airport(airport_ident)
);

make_function!(
    GetDeparturesAtAirport {
        airport_ident: String
    } => Vec<Departure> : get_departures_at_airport(airport_ident)
);

make_function!(
    GetArrivalsAtAirport {
        airport_ident: String
    } => Vec<Arrival> : get_arrivals_at_airport(airport_ident)
);

make_function!(
    GetApproachesAtAirport {
        airport_ident: String
    } => Vec<Approach> : get_approaches_at_airport(airport_ident)
);

make_function!(
    GetWaypointsAtAirport {
        airport_ident: String
    } => Vec<Waypoint> : get_waypoints_at_airport(airport_ident)
);

make_function!(
    GetNdbNavaidsAtAirport {
        airport_ident: String
    } => Vec<NdbNavaid> : get_ndb_navaids_at_airport(airport_ident)
);

make_function!(
    GetGatesAtAirport {
        airport_ident: String
    } => Vec<Gate> : get_gates_at_airport(airport_ident)
);

make_function!(
    GetCommunicationsAtAirport {
        airport_ident: String
    } => Vec<Communication> : get_communications_at_airport(airport_ident)
);

make_function!(
    GetGlsNavaidsAtAirport {
        airport_ident: String
    } => Vec<GlsNavaid> : get_gls_navaids_at_airport(airport_ident)
);

make_function!(
    GetPathPointsAtAirport {
        airport_ident: String
    } => Vec<PathPoint> : get_path_points_at_airport(airport_ident)
);

/// Generates boilerplate code for wrapping async functions in a uniform interface.
///
/// This macro simplifies the process of exposing a set of structs that implement an async `run` method
/// (via a `Function` trait) into a single deserializable enum for runtime dispatch and execution.
///
/// # Example
///
/// ```rust
/// #[derive(Deserialize)]
/// pub struct Foo {
///     bar: String,
/// }
///
/// impl Function for Foo {
///     async fn run(&mut self) -> Result<()> {
///         // Do some work...
///         Ok(())
///     }
/// }
///
/// define_interface_functions!(Foo);
/// ```
///
/// The macro will generate:
///
/// - A `FooWrapper` struct that owns a future created from the `run` method.
/// - An `InterfaceFunction` enum with a variant for each provided type (e.g. `Foo(FooWrapper)`).
/// - Implementations for `Deserialize`, `run`, and `id` on `InterfaceFunction`.
///
/// # JSON Input Example
///
/// A JSON payload like the following:
///
/// ```json
/// {
///     "id": "1",
///     "function": "Foo",
///     "data": {
///         "bar": "baz"
///     }
/// }
/// ```
///
/// Will deserialize into `InterfaceFunction::Foo(FooWrapper)`, ready to be executed via `.run()`.
///
/// # Execution
///
/// Calling `run()` on an `InterfaceFunction` polls the underlying future once per call,
/// returning either:
/// - `RunStatus::InProgress` if the future isn’t complete yet.
/// - `RunStatus::Finished` if the future resolved.
///
/// This is useful in our environment as we need to yield back to the sim in order not to block the thread,
/// and we may have some functions that aren't able to resolve in a single frame.
///
/// Once the future resolves, the result is automatically serialized into a `FunctionResult` structure and
/// sent across the commbus using the `NAVIGRAPH_FunctionResult` event.
///
/// # Note
///
/// During JSON deserialization, the input is validated to ensure the following:
/// - The `id`, `function`, and `data` fields are present.
/// - The `function` field matches the name of a registered function.
/// - The `data` field can be successfully parsed into the corresponding function's expected input type.
macro_rules! define_interface_functions {
    ($($fn_name:ident),* $(,)?) => {
        paste::paste! {
            /// The return status from a call to `run` on a function
            pub enum RunStatus {
                InProgress,
                Finished,
            }

            /// The actual return status of a function
            #[derive(serde::Serialize)]
            enum FunctionStatus {
                Success,
                Error,
            }

            /// The structure of a function result to be passed on the commbus
            #[derive(serde::Serialize)]
            struct FunctionResult {
                id: String,
                status: FunctionStatus,
                data: Option<serde_json::Value>
            }

            $(
                /// An internal wrapper around a function
                pub struct [<$fn_name Wrapper>] {
                    id: String,
                    args: serde_json::Value,
                    future: futures_lite::future::BoxedLocal<anyhow::Result<serde_json::Value>>,
                }

                impl [<$fn_name Wrapper>] {
                    fn new(id: String, args: serde_json::Value) -> anyhow::Result<Self> {
                        let mut instance = $fn_name::new(args.clone())?;
                        // Create the future. Note that this does not start executing until we poll it
                        let future = Box::pin(async move {
                            let result = instance.run().await?;
                            Ok(serde_json::to_value(result)?)
                         });

                        Ok(Self { id, args, future })
                    }

                    fn run(&mut self) -> anyhow::Result<RunStatus> {
                        // We allow the function run to be async in order to wait for certain conditions.
                        // However, MSFS WASM modules are not multithreaded so we need to yield back to the main thread.
                        // We get around this by polling once per update, and then continuing to poll (if needed) in later updates.
                        match futures_lite::future::block_on(futures_lite::future::poll_once(&mut self.future)) {
                            Some(result) => {
                                match result {
                                    Ok(data) => {
                                        // Send the success result across the commbus
                                        let serialized = serde_json::to_string(&FunctionResult {
                                            id: self.id.clone(),
                                            status: FunctionStatus::Success,
                                            data: Some(serde_json::to_value(&data)?),
                                        })?;
                                        msfs::commbus::CommBus::call(
                                            "NAVIGRAPH_FunctionResult",
                                            &serialized,
                                            msfs::commbus::CommBusBroadcastFlags::All,
                                        );
                                        Ok(RunStatus::Finished)
                                    }
                                    Err(err) => {
                                        // Send the error result across the commbus
                                        let serialized = serde_json::to_string(&FunctionResult {
                                            id: self.id.clone(),
                                            status: FunctionStatus::Error,
                                            data: Some(serde_json::to_value(&err.to_string())?),
                                        })?;
                                        msfs::commbus::CommBus::call(
                                            "NAVIGRAPH_FunctionResult",
                                            &serialized,
                                            msfs::commbus::CommBusBroadcastFlags::All,
                                        );
                                        Err(err)
                                    }
                                }
                            },
                            None => Ok(RunStatus::InProgress),
                        }
                    }
                }
            )*

            /// The available functions in the navigation data interface
            pub enum InterfaceFunction {
                $( $fn_name([<$fn_name Wrapper>]), )*
            }

            impl<'de> serde::Deserialize<'de> for InterfaceFunction {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    #[derive(serde::Deserialize)]
                    struct Helper {
                        id: String,
                        function: String,
                        data: serde_json::Value,
                    }

                    let Helper { id, function, data } = Helper::deserialize(deserializer)?;

                    match function.as_str() {
                        $(
                            stringify!($fn_name) => {
                                let wrapper = [<$fn_name Wrapper>]::new(id, data).map_err(serde::de::Error::custom)?;

                                Ok(InterfaceFunction::$fn_name(wrapper))
                            },
                        )*
                        _ => Err(serde::de::Error::custom(format!("Unknown function: {}", function))),
                    }
                }
            }

            impl InterfaceFunction {
                /// Run the function
                pub fn run(&mut self) -> anyhow::Result<RunStatus> {
                    match self {
                        $( Self::$fn_name(wrapper) => wrapper.run(), )*
                    }
                }

                // Get a `BTreeMap` representation of the current function's call data (looks identical to what was called via the commbus)
                pub fn get_function_details(&self) -> std::collections::BTreeMap<String, serde_json::Value> {
                    match self {
                        $( Self::$fn_name(wrapper) => {
                            let mut details = std::collections::BTreeMap::new();
                            details.insert("id".to_string(), serde_json::Value::String(wrapper.id.clone()));
                            details.insert("function".to_string(), serde_json::Value::String(stringify!($fn_name).to_string()));
                            details.insert("data".to_string(), wrapper.args.clone());
                            details
                        }, )*
                    }
                }
            }
        }
    };
}

define_interface_functions!(
    DownloadNavigationData,
    GetNavigationDataInstallStatus,
    GetDatabaseInfo,
    ExecuteSQLQuery,
    GetAirport,
    GetWaypoints,
    GetVhfNavaids,
    GetNdbNavaids,
    GetAirways,
    GetAirwaysAtFix,
    GetAirportsInRange,
    GetWaypointsInRange,
    GetVhfNavaidsInRange,
    GetNdbNavaidsInRange,
    GetAirwaysInRange,
    GetControlledAirspacesInRange,
    GetRestrictiveAirspacesInRange,
    GetCommunicationsInRange,
    GetRunwaysAtAirport,
    GetDeparturesAtAirport,
    GetArrivalsAtAirport,
    GetApproachesAtAirport,
    GetWaypointsAtAirport,
    GetNdbNavaidsAtAirport,
    GetGatesAtAirport,
    GetCommunicationsAtAirport,
    GetGlsNavaidsAtAirport,
    GetPathPointsAtAirport
);
