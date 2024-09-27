use aws_lc_rs::digest::{digest, SHA256};
use rcgen::{CertificateParams, KeyPair, PKCS_RSA_SHA256};
use serde::{Deserialize, Serialize};
use tokio::fs::{create_dir_all, read, read_to_string, try_exists};

use std::env::consts::OS;
use std::path::PathBuf;

use crate::random::random_alias;
use crate::OurError;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PROTO_VERSION: &str = "2.0";

#[derive(Clone, Debug)]
pub struct Config {
    pub info: Info,
    pub tls_config: TlsConfig,
}

impl Config {
    pub async fn from_xdg() -> Result<Self, OurError> {
        // First load the certificates, because the fingerprint is used in Info
        let tls_config = TlsConfig::from_xdg().await?;
        let info = Info::from_xdg(&tls_config.fingerprint).await?;

        Ok(Config { info, tls_config })
    }
}

#[derive(Clone, Debug)]
pub struct TlsConfig {
    pub public_pem: Vec<u8>,
    pub private_pem: Vec<u8>,
    pub fingerprint: String,
}

impl TlsConfig {
    pub async fn from_xdg() -> Result<Self, OurError> {
        let cfg_dir = cfg_dir().await?;

        let (public_pem, private_pem) = match (
            read(cfg_dir.join("public.pem")).await,
            read(cfg_dir.join("private.pem")).await,
        ) {
            (Ok(public), Ok(private)) => (public, private),
            _ => {
                // Generate new keypair because reading from disk failed
                // TODO: error
                let keypair = KeyPair::generate_for(&PKCS_RSA_SHA256).unwrap();
                let cert = CertificateParams::new([])
                    .unwrap()
                    .self_signed(&keypair)
                    .unwrap();
                (
                    cert.pem().as_bytes().to_vec(),
                    keypair.serialize_pem().as_bytes().to_vec(),
                )
            }
        };

        // let fingerprint = String::from_utf8_lossy(digest(&SHA256, &public_pem).as_ref()).to_string();
        // let fingerprint = format!("{:x?}", digest(&SHA256, &public_pem).as_ref());
        let fingerprint = digest(&SHA256, &public_pem)
            .as_ref()
            .iter()
            .map(|x| format!("{x:x}"))
            .collect();

        Ok(TlsConfig {
            public_pem,
            private_pem,
            fingerprint,
        })
    }
}

pub async fn cfg_dir() -> Result<PathBuf, OurError> {
    let cfg_dir = if let Some(cfg_dir) = dirs::config_dir() {
        cfg_dir.join(PKG_NAME)
    } else {
        return Err(OurError::NoXDG);
    };

    create_dir_all(&cfg_dir).await?;

    Ok(cfg_dir)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedConfig {
    pub alias: String,
    pub fingerprint: String,
}

impl SavedConfig {
    pub async fn from_xdg(fingerprint: &str) -> Result<Self, OurError> {
        let cfg_file = cfg_dir().await?.join("config.json");

        if let Ok(true) = try_exists(&cfg_file).await {
            let content = read_to_string(&cfg_file).await?;
            let saved_config: SavedConfig = serde_json::from_str(&content)?;

            if saved_config.fingerprint != fingerprint {
                panic!();
            }
            Ok(saved_config)
        } else {
            // Generate new identity and persist it
            // TODO: save to disk
            Ok(Self::new_random(fingerprint))
        }
    }

    pub fn new_random(fingerprint: &str) -> Self {
        Self {
            alias: random_alias(),
            fingerprint: fingerprint.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Info {
    #[serde(flatten)]
    pub config: SavedConfig,
    pub version: String,
    pub device_model: Option<String>,
    pub device_type: Option<crate::DeviceType>,
    pub port: u16,
    pub protocol: crate::Protocol,
    pub download: bool,
}

impl Info {
    pub async fn from_xdg(fingerprint: &str) -> Result<Self, OurError> {
        Ok(Self {
            config: SavedConfig::from_xdg(&fingerprint).await?,
            version: PROTO_VERSION.to_string(),
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
