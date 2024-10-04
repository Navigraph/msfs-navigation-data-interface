mod consts;
mod dispatcher;
mod download;
mod json_structs;
mod meta;
mod network_helper;
mod util;

use navigation_database::database::DatabaseV1;

#[msfs::gauge(name=navigation_data_interface)]
async fn navigation_data_interface(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    // Log the current version of the module
    println!(
        "{}",
        format!(
            "[NAVIGRAPH]: Navigation data interface version {} started",
            env!("CARGO_PKG_VERSION")
        )
    );
    let mut dispatcher: dispatcher::Dispatcher<'_> =
        dispatcher::Dispatcher::new(navigation_database::enums::InterfaceFormat::DFDv2);
    while let Some(event) = gauge.next_event().await {
        dispatcher.on_msfs_event(event);
    }

    Ok(())
}
