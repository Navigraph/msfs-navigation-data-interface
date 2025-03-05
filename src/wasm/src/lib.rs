use std::{env, sync::Arc, time::Instant};

use dotenvy_macro::dotenv;
use sentry_util::SentryTransportFactory;

mod consts;
mod dispatcher;
mod download;
mod json_structs;
mod meta;
mod network_helper;
mod sentry_util;
mod util;

const SENTRY_URL: &str = dotenv!("SENTRY_URL");

#[msfs::gauge(name=navigation_data_interface)]
async fn navigation_data_interface(
    mut gauge: msfs::Gauge,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut hash = env!("GIT_HASH").split_at(7).0.to_string();

    let version = match env!("MSFS_SDK").contains("msfs2020") {
        true => "2020",
        false => "2024",
    };

    hash = format!("{}-{}", version, hash);

    if option_env!("GITHUB_REPOSITORY").is_none() {
        hash = format!("{}-{}", hash, env!("CURRENT_TIME_ZULU"));
    }

    let release_name = format!("{}-{}", env!("CARGO_PKG_VERSION"), hash);

    // Log the current version of the module
    println!(
        "[NAVIGRAPH]: Navigation data interface version {} started",
        &release_name
    );

    let guard = sentry::init((
        SENTRY_URL,
        sentry::ClientOptions {
            release: Some(release_name.into()),
            transport: Some(Arc::new(SentryTransportFactory)),
            ..Default::default()
        },
    ));

    // sentry::capture_message("test message", sentry::Level::Fatal);

    let mut dispatcher = dispatcher::Dispatcher::new();
    let mut last_flush = Instant::now();
    while let Some(event) = gauge.next_event().await {
        dispatcher.on_msfs_event(event);
        if last_flush.elapsed().as_secs_f64() <= 30. {
            guard.flush(None);
            last_flush = Instant::now();
        }
    }

    Ok(())
}
