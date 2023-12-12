mod dispatcher;
mod download;
mod json_structs;
mod query;
mod util;

#[msfs::gauge(name=navdata_interface)]
async fn navdata_interface(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    let mut dispatcher = dispatcher::Dispatcher::new();
    while let Some(event) = gauge.next_event().await {
        dispatcher.on_msfs_event(event);
    }

    Ok(())
}
