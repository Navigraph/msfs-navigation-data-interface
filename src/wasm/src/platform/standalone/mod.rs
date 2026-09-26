use std::ptr::NonNull;

use anyhow::{Context, Result};
use rusqlite::{ffi, serialize::OwnedData, Connection, DatabaseName};

use super::Platform;
use crate::{
    database::CycleInfo,
    interface::{DownloadProgressEvent, InterfaceEvent},
};

/// The exported entry points, called by the `StandaloneTransport` in the TS package
mod exports;

/// The mock navigation data, generated inside the standalone build container (`bun run build:wasm:standalone`),
/// which sets `NAVIGRAPH_MOCK_DATA_DIR` to its output folder
const MOCK_NAVDATA: &[u8] = include_bytes!(concat!(
    env!(
        "NAVIGRAPH_MOCK_DATA_DIR",
        "The standalone build must run through `bun run build:wasm:standalone`, which generates the mock navigation data"
    ),
    "/mock-navdata.sqlite"
));

/// The cycle info of the mock navigation data
const MOCK_CYCLE_JSON: &str = include_str!(concat!(
    env!(
        "NAVIGRAPH_MOCK_DATA_DIR",
        "The standalone build must run through `bun run build:wasm:standalone`, which generates the mock navigation data"
    ),
    "/mock-cycle.json"
));

/// The path reported for the mock navigation data
const MOCK_NAVDATA_PATH: &str = "mock-navdata.sqlite";

/// The number of progress events sent by the mock download
const MOCK_DOWNLOAD_CHUNKS: usize = 4;

/// The size of each chunk of the mock download
const MOCK_DOWNLOAD_CHUNK_SIZE_BYTES: usize = 4 * 1024 * 1024;

mod host {
    #[link(wasm_import_module = "navigraph")]
    extern "C" {
        /// Deliver a message to the host. The host copies the data before returning.
        pub fn send_message(
            channel_ptr: *const u8,
            channel_len: usize,
            data_ptr: *const u8,
            data_len: usize,
        );
    }
}

/// The standalone platform: runs outside the sim, messaging the host directly and serving the embedded mock navigation data
pub(crate) struct StandalonePlatform;

impl Platform for StandalonePlatform {
    fn send_message(channel: &str, data: &str) {
        // SAFETY: The pointers are valid for the given lengths for the duration of the call, and the host copies the data before returning
        unsafe { host::send_message(channel.as_ptr(), channel.len(), data.as_ptr(), data.len()) }
    }

    fn init_database() -> Result<Option<Connection>> {
        Ok(Some(Self::open_database()?))
    }

    fn open_database() -> Result<Connection> {
        let mut conn = Connection::open_in_memory()?;

        // Load the embedded database into memory. SQLite takes ownership of the buffer, which must be allocated with sqlite3_malloc.
        // SAFETY: The buffer is allocated by SQLite with the required size, and fully initialized before being handed over
        let data = unsafe {
            let ptr = ffi::sqlite3_malloc64(MOCK_NAVDATA.len() as u64) as *mut u8;
            let ptr = NonNull::new(ptr).context("can't allocate memory for mock navigation data")?;
            std::ptr::copy_nonoverlapping(MOCK_NAVDATA.as_ptr(), ptr.as_ptr(), MOCK_NAVDATA.len());
            OwnedData::from_raw_nonnull(ptr, MOCK_NAVDATA.len())
        };
        conn.deserialize(DatabaseName::Main, data, true)
            .context("can't load mock navigation data")?;

        // Keep temp storage in memory, as there is no filesystem to rely on
        conn.execute_batch("PRAGMA temp_store = MEMORY")?;

        Ok(conn)
    }

    fn get_cycle_info() -> Result<CycleInfo> {
        CycleInfo::from_json(MOCK_CYCLE_JSON)
    }

    fn installed_path() -> Option<String> {
        Some(MOCK_NAVDATA_PATH.to_owned())
    }

    async fn get_latest_cycle() -> Result<Option<String>> {
        Ok(Some(Self::get_cycle_info()?.cycle))
    }

    /// Simulates a download by sending progress events. The mock navigation data is reloaded when the connection is reopened.
    async fn download_navigation_data(_url: &str) -> Result<()> {
        let total_bytes = MOCK_DOWNLOAD_CHUNKS * MOCK_DOWNLOAD_CHUNK_SIZE_BYTES;

        for i in 0..MOCK_DOWNLOAD_CHUNKS {
            InterfaceEvent::send_download_progress_event(DownloadProgressEvent {
                total_bytes,
                downloaded_bytes: i * MOCK_DOWNLOAD_CHUNK_SIZE_BYTES,
                current_chunk: i,
                total_chunks: MOCK_DOWNLOAD_CHUNKS,
            })
            .context("can't send download progress event")?;
        }

        Ok(())
    }

    async fn install_downloaded_navigation_data() -> Result<()> {
        Ok(())
    }
}
