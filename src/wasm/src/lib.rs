use std::env;

mod consts;
mod dispatcher;
mod download;
mod json_structs;
mod meta;
mod network_helper;
mod util;

#[msfs::gauge(name=navigation_data_interface)]
async fn navigation_data_interface(
    mut gauge: msfs::Gauge,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut hash = env!("GIT_HASH").split_at(7).0.to_string();

    if env::var("GITHUB_REPOSITORY").is_err() {
        let time = chrono::Utc::now();
        hash = format!(
            "{}-{}",
            hash,
            time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        );
    }

    // Log the current version of the module
    println!(
        "[NAVIGRAPH]: Navigation data interface version {}-{} started",
        env!("CARGO_PKG_VERSION"),
        hash
    );
    let mut dispatcher = dispatcher::Dispatcher::new();
    while let Some(event) = gauge.next_event().await {
        dispatcher.on_msfs_event(event);
    }

    Ok(())
}
