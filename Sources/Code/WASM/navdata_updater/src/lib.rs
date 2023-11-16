mod dispatcher;
mod download;

#[msfs::gauge(name=navdata_updater)]
async fn navdata_updater(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    let mut dispatcher = dispatcher::Dispatcher::new();
    while let Some(event) = gauge.next_event().await {
        dispatcher.on_msfs_event(event);
    }

    Ok(())
}
