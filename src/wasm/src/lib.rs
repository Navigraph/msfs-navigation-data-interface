mod consts;
mod dispatcher;
mod download;
mod json_structs;
mod util;

#[msfs::gauge(name=navigation_data_interface)]
async fn navigation_data_interface(
    mut gauge: msfs::Gauge,
) -> Result<(), Box<dyn std::error::Error>> {
    let hash = env!("GIT_HASH").split_at(7).0;

    // Log the current version of the module
    println!(
        "[NAVIGRAPH]: Navigation data interface version {}-{} started",
        env!("CARGO_PKG_VERSION"),
        hash
    );
    let mut dispatcher: dispatcher::Dispatcher<'_> =
        dispatcher::Dispatcher::new(navigation_database::enums::InterfaceFormat::DFDv2);
    while let Some(event) = gauge.next_event().await {
        dispatcher.on_msfs_event(event);
    }

    Ok(())
}
