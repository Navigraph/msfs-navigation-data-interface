//! The platform adapter layer.
//!
//! The core (database, queries, function dispatch) only talks to the platform through the [`Platform`] trait. The implementation is
//! selected at compile time with the `msfs` or `standalone` feature, and used through the [`ActivePlatform`] alias (static dispatch).

use anyhow::Result;
use rusqlite::Connection;

use crate::database::CycleInfo;

/// The MSFS gauge: CommBus, work folder, bundled data and network downloads
#[cfg(feature = "msfs")]
mod msfs;
/// Runs outside the sim, serving mock navigation data generated in the standalone build container
#[cfg(feature = "standalone")]
mod standalone;

#[cfg(feature = "msfs")]
pub(crate) type ActivePlatform = msfs::MsfsPlatform;
#[cfg(feature = "standalone")]
pub(crate) type ActivePlatform = standalone::StandalonePlatform;

/// Everything the core needs from the environment it runs in
pub(crate) trait Platform {
    /// Send a message to the JS side
    ///
    /// * `channel` - The channel to send the message on, such as `NAVIGRAPH_Event`
    /// * `data` - The serialized message
    fn send_message(channel: &str, data: &str);

    /// Prepare the navigation data to use at startup and open it. Returns `None` if there is no navigation data available.
    fn init_database() -> Result<Option<Connection>>;

    /// Open the active navigation data, e.g. after a download
    fn open_database() -> Result<Connection>;

    /// Get the cycle info of the active navigation data
    fn get_cycle_info() -> Result<CycleInfo>;

    /// Get the path of the active navigation data, as reported to the JS side
    fn installed_path() -> Option<String>;

    /// Get the latest available cycle, or `None` if it can't be determined (e.g. when offline)
    async fn get_latest_cycle() -> Result<Option<String>>;

    /// Download navigation data from the given URL, sending `DownloadProgress` events. Must not affect the active navigation data.
    ///
    /// * `url` - A signed URL to download the navigation data from
    async fn download_navigation_data(url: &str) -> Result<()>;

    /// Replace the active navigation data with the downloaded data. Called while the database connection is closed.
    async fn install_downloaded_navigation_data() -> Result<()>;
}
