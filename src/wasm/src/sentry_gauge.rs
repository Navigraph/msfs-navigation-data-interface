use std::{
    cell::LazyCell,
    fs::OpenOptions,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use dotenv_codegen::dotenv;
use msfs::{
    network::{NetworkRequest, NetworkRequestBuilder, NetworkRequestState},
    MSFSEvent,
};
use sentry::integrations::anyhow::capture_anyhow;
use serde::{Deserialize, Serialize};

/// The path to the cache file
const SENTRY_POOL_FILE: &str = "\\work/ng_sentry_pool.json";

// The amount of seconds between forced Sentry flushes
const SENTRY_FLUSH_INTERVAL_SECONDS: u64 = 60;

// The global Sentry pool instance
const SENTRY_POOL: LazyCell<Mutex<SentryReportPool>> =
    LazyCell::new(|| Mutex::new(SentryReportPool::load().unwrap()));

// A pending sentry report
#[derive(Deserialize, Serialize)]
struct PendingSentryReport {
    url: String,
    auth: String,
    data: String,
    #[serde(skip_serializing, skip_deserializing)]
    request: Option<NetworkRequest>,
}

impl PendingSentryReport {
    /// Create a new report
    ///
    /// * `url` - The URL to post to
    /// * `auth` - The auth header value
    /// * `data` - The data to post
    pub fn new(url: String, auth: String, data: String) -> Self {
        Self {
            url,
            auth,
            data,
            request: None,
        }
    }

    /// Send the request to Sentry
    pub fn send(&mut self) -> Result<()> {
        let res = NetworkRequestBuilder::new(&self.url)
            .unwrap()
            .with_header(&format!("X-Sentry-Auth: {}", self.auth))
            .unwrap()
            .post(&self.data)
            .ok_or(anyhow!("Could not send request"))?;

        self.request.replace(res);

        Ok(())
    }
}

/// A "pool" of all outgoing sentry requests.
///
/// On panic, network requests aren't able to go out. This solves that issue by storing all pending requests to the file system and retrying them on next load
#[derive(Default, Deserialize, Serialize)]
struct SentryReportPool {
    reports: Vec<PendingSentryReport>,
}

impl SentryReportPool {
    /// Load the pool
    pub fn load() -> Result<Self> {
        // Read from the local file. If that fails, it is likely that the file doesn't exist, so we can just create an empty pool
        let status = if let Ok(contents) = std::fs::read_to_string(SENTRY_POOL_FILE) {
            serde_json::from_str(&contents)?
        } else {
            Self::default()
        };

        Ok(status)
    }

    /// Flush all pending requests to the file system
    pub fn flush(&self) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(SENTRY_POOL_FILE)?;

        serde_json::to_writer(file, &self)?;

        Ok(())
    }

    /// Save a report to the pool
    ///
    /// * `url` - The URL to post to
    /// * `auth` - The auth header value
    /// * `data` - The data to post
    pub fn save_report(&mut self, url: String, auth: String, data: String) -> Result<()> {
        let report = PendingSentryReport::new(url, auth, data);
        self.reports.push(report);
        self.flush()?;

        Ok(())
    }

    /// Get the number of current pending reports
    pub fn num_pending_reports(&self) -> usize {
        self.reports.len()
    }

    /// Main update callback for the pool.
    ///
    /// Note: This MUST be called every frame, otherwise we *will* miss state updates on requests as DataReady is only available for a single frame
    pub fn update(&mut self) -> Result<()> {
        self.reports.retain_mut(|r| {
            // Get the request in the report. If one does not exist, create a request
            let Some(request) = r.request else {
                // Create the request. If it fails, just drop this report from the pool as it's likely something is wrong
                return r.send().is_ok();
            };

            // Retain if the state is not data ready
            request.state() != NetworkRequestState::DataReady
        });

        self.flush()?;

        Ok(())
    }
}

impl Drop for SentryReportPool {
    fn drop(&mut self) {
        // Ensure we have the latest state reflected in the file system
        if let Err(e) = self.flush() {
            println!("[NAVIGRAPH]: Error on SentryReportPool drop: {e}");
        }
    }
}

/// The transport implementation for Sentry
struct MsfsSentryTransport {
    options: sentry::ClientOptions,
}

impl sentry::Transport for MsfsSentryTransport {
    fn send_envelope(&self, envelope: sentry::Envelope) {
        // Get the body
        let mut body = Vec::new();
        envelope.to_writer(&mut body).unwrap();

        // Get the URL and auth header
        let dsn = self.options.dsn.as_ref().unwrap();
        let user_agent = self.options.user_agent.clone();
        let auth = dsn.to_auth(Some(&user_agent)).to_string();
        let url = dsn.envelope_api_url().to_string();

        // Save to the pool
        if let Ok(mut pool) = SENTRY_POOL.try_lock() {
            match pool.save_report(
                url.clone(),
                auth.clone(),
                String::from_utf8(body.clone()).unwrap(),
            ) {
                Ok(_) => {}
                Err(e) => {
                    println!("[NAVIGRAPH]: Unable to cache Sentry report: {e}");
                    return;
                }
            };
        }
    }
}

/// The factory implementation for Sentry
struct MsfsSentryTransportFactory;

impl sentry::TransportFactory for MsfsSentryTransportFactory {
    fn create_transport(
        &self,
        options: &sentry::ClientOptions,
    ) -> std::sync::Arc<dyn sentry::Transport> {
        Arc::new(MsfsSentryTransport {
            options: options.clone(),
        })
    }
}

/// A trait that represents the interface for a Gauge that reports errors to Sentry
pub trait SentryGauge {
    fn initialize() -> Result<Self>
    where
        Self: Sized;
    fn update(&mut self) -> Result<()>;
}

/// Create a sentry "executor" around a gauge
///
/// Note: This MUST be the first function called within a gauge callback. Nothing will run after this
pub async fn wrap_gauge_with_sentry<T>(mut gauge: msfs::Gauge) -> Result<()>
where
    T: SentryGauge,
{
    // Wait out the first few events as it is unreliable to call initialization logic then
    while let Some(event) = gauge.next_event().await {
        if matches!(event, MSFSEvent::PostInstall) {
            break;
        }
    }

    // Initialize sentry
    let sentry_guard = sentry::init((
        dotenv!("SENTRY_URL"),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            transport: Some(Arc::new(MsfsSentryTransportFactory)),
            ..Default::default()
        },
    ));

    // Drain any pending reports
    if let Ok(mut pool) = SENTRY_POOL.try_lock() {
        while pool.num_pending_reports() > 0 {
            // Await next gauge event
            gauge.next_event().await;
            pool.update()?;
        }
    } else {
        return Err(anyhow!("Unable to lock SENTRY_POOL"));
    };

    // Create the gauge instance
    let mut instance = match T::initialize() {
        Ok(instance) => instance,
        Err(e) => {
            capture_anyhow(&e);
            return Err(e);
        }
    };

    // Keep track of when we flush
    let mut last_sentry_flush = Instant::now();

    // Finally, go to the main event loop
    while let Some(event) = gauge.next_event().await {
        // Ensure the event is what we are looking for
        let MSFSEvent::PreUpdate = event else {
            continue;
        };

        match instance.update() {
            Ok(_) => {}
            Err(e) => {
                capture_anyhow(&e);
                println!("[NAVIGRAPH]: Error encountered in update: {e}")
            }
        };

        // Flush sentry if interval has passed
        if last_sentry_flush.elapsed() >= Duration::from_secs(SENTRY_FLUSH_INTERVAL_SECONDS) {
            sentry_guard.flush(None);
            last_sentry_flush = Instant::now();
        }
    }

    Ok(())
}

/// A convenience macro to handle the gauge entrypoint and sentry wrapping around a struct that implements `SentryGauge`
///
/// Example: `sentry_gauge!(MyGauge, my_gauge_name)`
#[macro_export]
macro_rules! sentry_gauge {
    ($type_:ty, $name:expr) => {
        #[msfs::gauge(name = $name)]
        async fn main(gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
            wrap_gauge_with_sentry::<$type_>(gauge).await?;
            Ok(())
        }
    };
}
