// config.rs

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use thiserror::Error;

pub type SharedConfig = Arc<RwLock<Config>>;

const APP_NAME: &str = "sidereal";
const CONFIG_FILE_NAME: &str = "config.json";

fn default_config_path() -> PathBuf {
    let mut dir = dirs_next::config_dir().unwrap_or_else(|| {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
    });
    dir.push(APP_NAME);
    // ensure directory exists
    let _ = std::fs::create_dir_all(&dir);
    dir.push(CONFIG_FILE_NAME);
    dir
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub location: Location,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            location: Location {
                latitude: 45.503575,
                longitude: -73.587090,
                altitude: 100.0,
            },
        }
    }
}

impl Config {
    /// Load from disk, or return default if missing
    pub fn load_or_default() -> Result<Self, ConfigError> {
        let path = default_config_path();
        if path.exists() {
            let raw = fs::read_to_string(path)?;
            let cfg = serde_json::from_str(&raw)?;
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }

    /// Save current config to disk
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = default_config_path();
        let serialized = serde_json::to_string_pretty(self)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Initialize the global config at application startup
    pub fn initialize() -> Result<(), ConfigError> {
        let cfg = Config::load_or_default()?;
        let mut guard = GLOBAL_CONFIG.write().unwrap();
        *guard = cfg;
        Ok(())
    }

    /// Persist the in-memory global config to disk
    pub fn persist() -> Result<(), ConfigError> {
        let guard = GLOBAL_CONFIG.read().unwrap();
        guard.save()
    }

    /// Get a cloned snapshot of the current config
    pub fn get() -> Config {
        GLOBAL_CONFIG.read().unwrap().clone()
    }

    /// Example setter: update latitude synchronously and persist
    pub fn set_location(latitude: f32, longitude: f32, altitude: f32) -> Result<(), ConfigError> {
        {
            let mut guard = GLOBAL_CONFIG.write().unwrap();
            guard.location.latitude = latitude;
            guard.location.longitude = longitude;
            guard.location.altitude = altitude;
        }
        // save after modification
        Config::persist()
    }
}

/// Global shared config, accessible synchronously
pub static GLOBAL_CONFIG: Lazy<SharedConfig> =
    Lazy::new(|| Arc::new(RwLock::new(Config::default())));
