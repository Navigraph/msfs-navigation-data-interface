#[cfg(all(feature = "msfs", feature = "standalone"))]
compile_error!("The `msfs` and `standalone` features are mutually exclusive. Build the standalone module with `--no-default-features --features standalone`");
#[cfg(not(any(feature = "msfs", feature = "standalone")))]
compile_error!("Either the `msfs` or the `standalone` feature must be enabled");

/// SQLite mapping implementation
mod database;
/// Interface function definitions
mod funcs;
/// Function queue and events shared by all platforms
mod interface;
/// The platform adapters (MSFS gauge or standalone), selected at compile time
mod platform;
