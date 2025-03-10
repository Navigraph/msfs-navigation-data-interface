#[msfs::gauge(name=navigation_data_interface)]
async fn navigation_data_interface(
    mut gauge: msfs::Gauge,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(event) = gauge.next_event().await {}

    Ok(())
}
