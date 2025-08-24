// config.rs

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use crate::model::{SiderealError, SiderealResult};

pub type SharedConfig = Arc<RwLock<Config>>;

const APP_NAME: &str = "sidereal";
const CONFIG_FILE_NAME: &str = "config.json";

fn default_config_path() -> PathBuf {
    let mut dir = dirs_next::config_dir().unwrap_or_else(|| {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
    });
    let _ = std::fs::create_dir_all(&dir); // directory creation still sync for now
    dir.push(APP_NAME);
    let _ = std::fs::create_dir_all(&dir); // ensure ~/.config/sidereal exists
    dir.push(CONFIG_FILE_NAME);
    dir
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Location {
    pub latitude: f32,
    pub longitude: f32,
    pub altitude: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CameraConfig {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub location: Location,
    pub server: Option<String>,
    pub cameras: Vec<CameraConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            location: Location {
                latitude: 45.503575,
                longitude: -73.587090,
                altitude: 100.0,
            },
            server: None,
            cameras: vec![],
        }
    }
}

impl Config {
    /// Load from disk, or return default if missing
    pub async fn load_or_default() -> SiderealResult<Config> {
        let path = default_config_path();
        if path.exists() {
            let raw = tokio::fs::read_to_string(path)
                .await
                .map_err(|e| SiderealError::ConfigError(e.to_string()))?;
            let cfg = serde_json::from_str(&raw)
                .map_err(|e| SiderealError::ConfigError(e.to_string()))?;
            Ok(cfg)
        } else {
            Ok(Config::default())
        }
    }

    /// Save current config to disk
    pub async fn save(&self) -> SiderealResult<()> {
        let path = default_config_path();
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| SiderealError::ConfigError(e.to_string()))?;
        tokio::fs::write(path, serialized)
            .await
            .map_err(|e| SiderealError::ConfigError(e.to_string()))?;
        Ok(())
    }

    /// Initialize the global config at application startup
    pub async fn initialize() -> SiderealResult<()> {
        let cfg = Config::load_or_default().await?;
        let mut guard = GLOBAL_CONFIG.write().await;
        *guard = cfg;
        Ok(())
    }

    /// Persist the in-memory global config to disk
    pub async fn persist() -> SiderealResult<()> {
        let guard = GLOBAL_CONFIG.read().await;
        guard
            .save()
            .await
            .map_err(|e| SiderealError::ConfigError(e.to_string()))
    }

    /// Get a cloned snapshot of the current config
    pub async fn get() -> Config {
        GLOBAL_CONFIG.read().await.clone()
    }

    /// Example setter: update location and persist
    pub async fn set_location(latitude: f32, longitude: f32, altitude: f32) -> SiderealResult<()> {
        {
            let mut guard = GLOBAL_CONFIG.write().await;
            guard.location.latitude = latitude;
            guard.location.longitude = longitude;
            guard.location.altitude = altitude;
        }
        Config::persist().await
    }
}

/// Global shared config, accessible asynchronously
pub static GLOBAL_CONFIG: Lazy<SharedConfig> =
    Lazy::new(|| Arc::new(RwLock::new(Config::default())));
