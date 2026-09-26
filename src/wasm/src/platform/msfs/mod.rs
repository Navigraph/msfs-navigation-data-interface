use std::{
    cmp::Ordering,
    fs::{self, read_dir, File, OpenOptions},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use ::msfs::{
    commbus::{CommBus, CommBusBroadcastFlags},
    network::NetworkRequestBuilder,
};
use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use zip::ZipArchive;

use self::futures::AsyncNetworkRequest;
use super::Platform;
use crate::{
    database::CycleInfo,
    interface::{DownloadProgressEvent, InterfaceEvent},
};

/// Developer-configured values for interface
mod config;
/// Futures implementations for use in interface functions
mod futures;
/// The MSFS gauge entrypoint
mod gauge;
/// The sentry wrapper implementation around the MSFS gauge callbacks
mod sentry_gauge;

/// The path to the navigation data files folder in the work directory
const WORK_NAVIGATION_DATA_FOLDER: &str = "\\work/NavigationData";
/// The path to the "master" cycle info JSON
const WORK_CYCLE_JSON_PATH: &str = "\\work/NavigationData/cycle.json";
/// The path to the "master" SQLite DB
const WORK_DB_PATH: &str = "\\work/NavigationData/db.s3db";
/// The folder name for bundled navigation data
const BUNDLED_FOLDER_NAME: &str = ".\\Navigraph/BundledData";

/// The URL to get the latest available cycle number
const LATEST_CYCLE_ENDPOINT: &str = "https://navdata.api.navigraph.com/info";

/// The path to the temporary download file
const DOWNLOAD_TEMP_FILE_PATH: &str = "\\work/ng_download.temp";

/// The max size in bytes of each request during the download function (set to 4MB curently)
const DOWNLOAD_CHUNK_SIZE_BYTES: usize = 4 * 1024 * 1024;

/// The return type from the latest cycle endpoint
#[derive(Deserialize)]
struct CycleResponseInfo {
    cycle: String,
}

/// Find the bundled navigation data distribution
fn get_bundled_db() -> Result<Option<DatabaseDistributionInfo>> {
    let bundled_entries = match read_dir(BUNDLED_FOLDER_NAME) {
        Ok(dir) => dir.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(_) => return Ok(None),
    };

    // Try finding cycle.json
    let Some(cycle_file_name) = bundled_entries
        .iter()
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_owned()))
        .find(|e| *e == String::from("cycle.json"))
    else {
        return Ok(None);
    };

    // Try finding the DB (we don't know the full filename, only extension)
    let Some(db_file_name) = bundled_entries
        .iter()
        .filter_map(|e| e.file_name().to_str().map(|s| s.to_owned()))
        .find(|e| e.ends_with(".s3db"))
    else {
        return Ok(None);
    };

    Ok(Some(DatabaseDistributionInfo::new(
        Path::new(&format!(".\\{BUNDLED_FOLDER_NAME}\\{cycle_file_name}")), // We need to reconstruct the bundled path to include the proper syntax to reference non-work folder files
        Path::new(&format!(".\\{BUNDLED_FOLDER_NAME}\\{db_file_name}")),
    )?))
}

/// A pair of a cycle info JSON and the corresponding SQLite database.
struct DatabaseDistributionInfo {
    cycle_info: CycleInfo,
    db_path: PathBuf,
    cycle_info_path: PathBuf,
}

impl DatabaseDistributionInfo {
    /// Create a new distribution info set
    ///
    /// * `cycle_info_path` - The path to the cycle info JSON
    /// * `db_path` - The path to the SQLite DB
    pub fn new(cycle_info_path: &Path, db_path: &Path) -> Result<Self> {
        // Ensure paths exist (fs::exists is unreliable, so try getting a handle)
        if File::open(cycle_info_path).is_err() || File::open(db_path).is_err() {
            return Err(anyhow!("invalid distribution path"));
        }

        Ok(Self {
            cycle_info: CycleInfo::from_path(cycle_info_path)?,
            db_path: db_path.to_owned(),
            cycle_info_path: cycle_info_path.to_owned(),
        })
    }
}

/// The MSFS platform: CommBus messaging, navigation data in the work folder (bundled or downloaded), and network downloads
pub(crate) struct MsfsPlatform;

impl Platform for MsfsPlatform {
    fn send_message(channel: &str, data: &str) {
        CommBus::call(channel, data, CommBusBroadcastFlags::All);
    }

    /// Try to load a database (either bundled or downloaded)
    ///
    /// This searches for the best DB to use by comparing the cycle and revision of both the downloaded (in work folder) and bundled navigation data.
    fn init_database() -> Result<Option<Connection>> {
        // Get distribution info of both bundled and downloaded DBs, if they exist
        let bundled_distribution = get_bundled_db()?;
        let downloaded_distribution =
            DatabaseDistributionInfo::new(Path::new(WORK_CYCLE_JSON_PATH), Path::new(WORK_DB_PATH))
                .ok();

        // Find the most recent distribution
        let latest = [downloaded_distribution, bundled_distribution]
            .into_iter()
            .filter_map(|d| d)
            .reduce(|a, b| {
                // First, compare by cycle number
                match a
                    .cycle_info
                    .cycle
                    .parse::<u32>()
                    .unwrap_or(0)
                    .cmp(&b.cycle_info.cycle.parse::<u32>().unwrap_or(0))
                {
                    Ordering::Greater => a,
                    Ordering::Less => b,
                    Ordering::Equal => {
                        // If they are somehow equal, compare revisions
                        match a
                            .cycle_info
                            .revision
                            .parse::<u32>()
                            .unwrap_or(0)
                            .cmp(&b.cycle_info.revision.parse::<u32>().unwrap_or(0))
                        {
                            Ordering::Greater | Ordering::Equal => a,
                            Ordering::Less => b,
                        }
                    }
                }
            });

        // If we somehow don't have a cycle in bundled or downloaded, return an empty instance
        let Some(latest) = latest else {
            return Ok(None);
        };

        // Ensure parent folder exists (ignore the result as it will return an error if it already exists)
        let _ = fs::create_dir_all(WORK_NAVIGATION_DATA_FOLDER);

        // Ensure files get copied over
        if latest.cycle_info_path != PathBuf::from(WORK_CYCLE_JSON_PATH) {
            fs::copy(&latest.cycle_info_path, WORK_CYCLE_JSON_PATH)?;
        }
        if latest.db_path != PathBuf::from(WORK_DB_PATH) {
            fs::copy(&latest.db_path, WORK_DB_PATH)?;
        }

        // The only way this can fail (since we know now that the path is valid) is if the file is corrupt, in which case we should report to sentry
        Ok(Some(Self::open_database()?))
    }

    fn open_database() -> Result<Connection> {
        // We have to open with flags because the SQLITE_OPEN_CREATE flag with the default open causes the file to
        // be overwritten
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;

        // The WORK_DB_PATH is the "master" SQLite path. We have logic copying over bundled navigation data if needed in init_database.
        let conn = Connection::open_with_flags(WORK_DB_PATH, flags)?;

        // Use memory for temp storage (avoids directory issues with the work folder, with the tradeoff of higher memory usage for queries)
        conn.execute_batch("PRAGMA temp_store = MEMORY")?;

        Ok(conn)
    }

    fn get_cycle_info() -> Result<CycleInfo> {
        // The WORK_CYCLE_JSON_PATH is the "master" cycle JSON path.
        CycleInfo::from_path(Path::new(WORK_CYCLE_JSON_PATH))
    }

    fn installed_path() -> Option<String> {
        Some(WORK_DB_PATH.to_owned())
    }

    async fn get_latest_cycle() -> Result<Option<String>> {
        // Try to get the latest available cycle from our API. Support cases in which the user may be offline by returning a None instead
        let latest_cycle = if let Ok(res) = NetworkRequestBuilder::new(LATEST_CYCLE_ENDPOINT)
            .context("can't create new NetworkRequestBuilder")?
            .get()
            .context(".get() returned None")?
            .wait_for_data()
            .await
        {
            let response_info = serde_json::from_slice::<CycleResponseInfo>(&res)
                .context("can't deserialize cycle response info")?;

            Some(response_info.cycle)
        } else {
            None
        };

        Ok(latest_cycle)
    }

    /// Download the navigation data zip file to the temp file location
    async fn download_navigation_data(url: &str) -> Result<()> {
        // Figure out total size of download (this request is acting like a HEAD since we don't have those in this environment. Nothing actually gets downloaded since we are constraining the range)
        let request = NetworkRequestBuilder::new(url)
            .context("can't create new NetworkRequestBuilder")?
            .with_header(&format!("Range: bytes=0-0"))
            .context(".with_header() returned None")?
            .get()
            .context(".get() returned None")?;

        request
            .wait_for_data()
            .await
            .context("can't wait for head request data")?;

        // Try parsing the content-range header
        let total_bytes = request
            .header_section("content-range")
            .context("no content-range header")?
            .trim()
            .split("/")
            .last()
            .context("invalid content-range")?
            .parse::<usize>()
            .context("can't parse content-range total bytes")?;

        // Total amount of chunks to download.  We need to download the data in chunks of DOWNLOAD_CHUNK_SIZE_BYTES to avoid a timeout, so we need to keep track of a "working" accumulation of all responses
        let total_chunks = total_bytes.div_ceil(DOWNLOAD_CHUNK_SIZE_BYTES);

        // Store the download to a file to avoid holding in-memory
        let mut download_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(DOWNLOAD_TEMP_FILE_PATH)
            .context("can't open temp download file")?;

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
            })
            .context("can't send download progress event")?;

            // Dispatch the request
            let data = NetworkRequestBuilder::new(url)
                .context("can't create new NetworkRequestBuilder")?
                .with_header(&format!("Range: bytes={range_start}-{range_end}"))
                .context(".with_header() returned None")?
                .get()
                .context(".get() returned None")?
                .wait_for_data()
                .await
                .context("can't wait for chunk request data")?;

            // Write to limit how much data we hold in memory at a time (will be a max of DOWNLOAD_CHUNK_SIZE_BYTES)
            download_file
                .write_all(&data)
                .context("can't write chunk to temp download file")?;
        }

        Ok(())
    }

    /// Extract the navigation data files from the zip file located in the temp location
    async fn install_downloaded_navigation_data() -> Result<()> {
        // Load the zip archive
        let zip_file =
            File::open(DOWNLOAD_TEMP_FILE_PATH).context("can't open temp download file")?;
        let mut zip = ZipArchive::new(BufReader::new(zip_file))
            .context("can't read zip archive from temp download file")?;

        // Ensure parent folder exists (ignore the result as it will return an error if it already exists)
        let _ = fs::create_dir_all(WORK_NAVIGATION_DATA_FOLDER);

        // Write the cycle.json file
        let mut cycle_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(WORK_CYCLE_JSON_PATH)
            .context("can't open cycle.json work path")?;

        let mut zip_cycle = zip
            .by_name("cycle.json")
            .context("can't find cycle.json in zip")?;
        std::io::copy(&mut zip_cycle, &mut cycle_file)
            .context("can't copy cycle.json from zip to work path")?;
        drop(zip_cycle);

        // Write the db file
        let db_name = zip
            .file_names()
            .find(|f| f.to_lowercase().ends_with(".s3db"))
            .context("unable to find sqlite db in downloaded zip")?
            .to_owned();

        let mut db_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(WORK_DB_PATH)
            .context("can't open db work path")?;

        let mut zip_db = zip.by_name(&db_name).context("can't find db in zip")?;
        std::io::copy(&mut zip_db, &mut db_file)
            .context("can't copy db from zip to work path")?;
        drop(zip_db);
        drop(zip);

        // Remove the temp file
        fs::remove_file(DOWNLOAD_TEMP_FILE_PATH).context("can't remove temp download file")?;

        Ok(())
    }
}
