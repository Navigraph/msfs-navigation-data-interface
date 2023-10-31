use msfs::{commbus, MSFSEvent};

fn download_navdata(json: &[u8]) {
    commbus::CommBus::call("NavdataUpdaterReceived", &[], commbus::CommBusBroadcastFlags::JS);
}

#[msfs::gauge(name=NAVDATA_UPDATER)]
async fn navdata_updater(mut gauge: msfs::Gauge) -> Result<(), Box<dyn std::error::Error>> {
    println!("mhm");
    while let Some(event) = gauge.next_event().await {
        match event {
            MSFSEvent::PostInitialize => {
                commbus::CommBus::register("DownloadNavdata", download_navdata);
            }
            _ => {}
        }
    }

    Ok(())
}