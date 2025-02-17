use aws_lc_rs::digest::{digest, SHA256};
use rcgen::{CertificateParams, KeyPair, PKCS_RSA_SHA256};
use serde::{Deserialize, Serialize};
use tokio::fs::{create_dir_all, read_to_string, write};

use std::env::consts::OS;
use std::path::PathBuf;

use crate::random::{alias_from_seed, random_alias};
use crate::OurError;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PROTO_VERSION: &str = "2.0";

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub info: Info,
    pub tls_config: TlsConfig,
}

impl Config {
    pub async fn from_xdg() -> Result<Self, OurError> {
        // First load the certificates, because the fingerprint is used in Info
        let tls_config = TlsConfig::from_xdg().await?;
        let info = Info::from_xdg(&tls_config.fingerprint).await?;

        info!("Operating as {}", info.config.alias);

        Ok(Config { info, tls_config })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TlsConfig {
    pub public_pem: Vec<u8>,
    pub private_pem: Vec<u8>,
    pub fingerprint: String,
}

impl TlsConfig {
    pub async fn from_xdg() -> Result<Self, OurError> {
        let cfg_dir = cfg_dir().await?;

        let private_key_file = cfg_dir.join("private.pem");

        let (keypair, private_pem) =
            if let Ok(private_key) = read_to_string(&private_key_file).await {
                debug!("Loading TLS private key from disk...");
                let keypair = KeyPair::from_pem(&private_key)?;
                let private_pem = keypair.serialize_pem().as_bytes().to_vec();
                (keypair, private_pem)
            } else {
                debug!("Generating new TLS private key...");
                let keypair = KeyPair::generate_for(&PKCS_RSA_SHA256)?;
                let private_pem = keypair.serialize_pem().as_bytes().to_vec();

                debug!("Persisting TLS private key to disk...");
                write(&private_key_file, &private_pem).await?;

                (keypair, private_pem)
            };

        let cert = CertificateParams::new([])
            .unwrap()
            .self_signed(&keypair)
            .unwrap();

        let public_pem = cert.pem().as_bytes().to_vec();

        let fingerprint = digest(&SHA256, pem::parse(&cert.pem()).unwrap().contents())
            .as_ref()
            .iter()
            .map(|x| format!("{x:x}"))
            .collect();

        debug!("Now operating with TLS fingerprint {}", fingerprint);

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SavedConfig {
    pub alias: String,
    pub fingerprint: String,
}

impl SavedConfig {
    /// Create a specific identity (alias/fingerprint)
    pub fn new(alias: &str, fingerprint: &str) -> Self {
        Self {
            alias: alias.to_string(),
            fingerprint: fingerprint.to_string(),
        }
    }

    /// Generate a random alias with a specific fingerprint
    pub fn new_random(fingerprint: &str) -> Self {
        Self {
            alias: random_alias(),
            fingerprint: fingerprint.to_string(),
        }
    }

    /// Derive a stable alias from a specific fingerprint
    pub fn from_fingerprint(fingerprint: &str) -> Self {
        Self {
            alias: alias_from_seed(fingerprint),
            fingerprint: fingerprint.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
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
            config: SavedConfig::from_fingerprint(&fingerprint),
            version: PROTO_VERSION.to_string(),
            device_model: Some(OS.to_string()),
            device_type: Some(DeviceType::default()),
            port: 53317,
            protocol: Protocol::Http,
            download: true,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Https,
}
