use std::fs::File;

use serde::Deserialize;

/// The path to an optional addon-specific config file containing data about the addon
const ADDON_CONFIG_FILE: &str = ".\\Navigraph/config.json";

/// Information about the current addon
#[derive(Deserialize)]
pub struct Addon {
    pub developer: String,
    pub product: String,
}

/// Configuration data provided by the developer
#[derive(Deserialize)]
pub struct Config {
    pub addon: Addon,
}

impl Config {
    /// Try to get the config
    pub fn get_config() -> Option<Self> {
        if let Ok(file) = File::open(ADDON_CONFIG_FILE) {
            serde_json::from_reader(file).ok()
        } else {
            None
        }
    }
}
