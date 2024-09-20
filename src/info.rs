use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Info {
    pub alias: String,
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<crate::DeviceType>,
    pub fingerprint: String,
    pub port: u16,
    pub protocol: crate::Protocol,
    pub download: bool,
}

#[derive(Debug, Serialize, Deserialize)]
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
