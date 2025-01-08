mod consts;
mod dispatcher;
mod download;
mod json_structs;
mod meta;
mod network_helper;
mod util;

#[msfs::gauge(name=navigation_data_interface)]
async fn navigation_data_interface(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    let hash = env!("GIT_HASH").split_at(7).0;

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
