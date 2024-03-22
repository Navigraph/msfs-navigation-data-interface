mod consts;
mod dispatcher;
mod download;
mod json_structs;
mod util;

#[msfs::gauge(name=navigation_data_interface)]
async fn navigation_data_interface(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    let mut dispatcher = dispatcher::Dispatcher::new();
    while let Some(event) = gauge.next_event().await {
        dispatcher.on_msfs_event(event);
    }

    Ok(())
}
