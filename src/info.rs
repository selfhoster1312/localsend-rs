use serde::{Serialize, Deserialize};
use tokio::fs::{create_dir_all, read_to_string, try_exists};

use std::env::consts::OS;

use crate::OurError;
use crate::random::random_alias;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedConfig {
    pub alias: String,
    pub fingerprint: String,
}

impl SavedConfig {
    pub async fn from_xdg() -> Result<Self, OurError> {
        let cfg_dir = if let Some(cfg_dir) = dirs::config_dir() {
            cfg_dir.join(PKG_NAME)
        } else {
            return Err(OurError::NoXDG);
        };

        create_dir_all(&cfg_dir).await?;

        let cfg_file = cfg_dir.join("config.json");

        if let Ok(true) = try_exists(&cfg_file).await {
            let content = read_to_string(&cfg_file).await?;
            Ok(serde_json::from_str(&content)?)
        } else {
            // Generate new identity and persist it
            // TODO: save to disk
            Ok(Self::new_random())
        }
    }

    pub fn new_random() -> Self {
        Self {
            alias: random_alias(),
            fingerprint: "BULLSHIT TODO".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Info {
    #[serde(flatten)]
    // TODO: saving TLS certificate/key
    pub config: SavedConfig,
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<crate::DeviceType>,
    pub port: u16,
    pub protocol: crate::Protocol,
    pub download: bool,
}

impl Info {
    pub async fn from_xdg() -> Result<Self, OurError> {
        Ok(Self {
            config: SavedConfig::from_xdg().await?,
            version: VERSION.to_string(),
            device_model: Some(OS.to_string()),
            device_type: Some(DeviceType::default()),
            port: 53317,
            protocol: Protocol::Http,
            download: true,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
}

impl Default for DeviceType {
    fn default() -> Self {
        if cfg!(target_os = "ios") || cfg!(target_os = "android") {
            Self::Mobile
        } else {
            Self::Desktop
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Https,
}

