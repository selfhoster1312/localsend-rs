use axum_server::tls_rustls::RustlsConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::{TcpListener, UdpSocket};

pub mod axum2;
mod error;
pub use error::OurError;
pub mod info;
pub mod random;

use info::{Config, DeviceType, Info, Protocol};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Announce {
    #[serde(flatten)]
    info: Info,
    announce: bool,
}

pub struct LocalSend {
    /// Information about the current device/session
    config: Config,
}

impl LocalSend {
    pub async fn new(config: Config) -> Result<LocalSend, OurError> {
        let config2 = config.clone();

        println!("Spawning web task");

        tokio::task::spawn(async {
            // TODO: add IPv6
            let listener = TcpListener::bind(format!("0.0.0.0:{}", config2.info.port))
                .await
                .unwrap();
            Self::blocking_recv_web(listener, config2).await.unwrap();
        });

        println!("Done");
        println!("Spawning multicast task");

        let config2 = config.clone();
        tokio::task::spawn(async {
            // TODO: add IPv6
            let listener = UdpSocket::bind("224.0.0.167:53317").await.unwrap();
            Self::send_announce(&listener, config2.info.clone())
                .await
                .unwrap();
            Self::blocking_recv_multicast(listener, config2)
                .await
                .unwrap();
        });

        println!("Done");

        Ok(LocalSend { config })
    }

    /// Wait for web requests on the configured HTTP port
    ///
    /// This function, although async, will occupy the current task and should be spawned
    /// on a dedicated task. This is done automatically by [`LocalSend::new`].
    pub async fn blocking_recv_web(listener: TcpListener, config: Config) -> Result<(), OurError> {
        // TODO: error
        let rustls_config =
            RustlsConfig::from_pem(config.tls_config.public_pem, config.tls_config.private_pem)
                .await
                .unwrap();

        // TODO: is this ok to turn tokio::TcpListener into std::TcpListener for axum_server???
        axum_server::from_tcp_rustls(listener.into_std().unwrap(), rustls_config)
            .serve(crate::axum2::route(config.info).into_make_service())
            .await
            .unwrap();

        unreachable!();
    }

    // TODO: not receiving anything?
    // probably doing it all wrong
    pub async fn blocking_recv_multicast(
        socket: UdpSocket,
        _config: Config,
    ) -> Result<(), OurError> {
        // let _ = socket.set_broadcast(true);

        let mut buf = [0; 4096];
        while let Ok(size) = socket.recv(&mut buf).await {
            println!("Received response");
            if let Ok(response) = serde_json::from_slice::<Info>(&buf[0..size]) {
                println!(
                    "Received LAN advertisement response for LocalSend client: {}",
                    response.config.alias
                );
            } else if let Ok(response) = serde_json::from_slice::<Announce>(&buf[0..size]) {
                println!(
                    "Received LAN advertisement request for LocalSend client: {}",
                    response.info.config.alias
                );
                // TODO: we should probably reply to this announecment
            }
        }

        unreachable!();
    }

    pub async fn from_xdg() -> Result<LocalSend, OurError> {
        Self::new(Config::from_xdg().await?).await
    }

    pub async fn send_announce(socket: &UdpSocket, info: Info) -> Result<(), OurError> {
        println!("Announcing to the network...");

        let announce = Announce {
            announce: true,
            info: info,
        };
        let json = serde_json::to_string(&announce)?;
        println!("{json}");

        socket.send_to(json.as_bytes(), "224.0.0.167:53317").await?;

        Ok(())
    }

    pub async fn send(
        &self,
        recipient: &str,
        file_type: impl Into<String>,
        data: Vec<u8>,
    ) -> Result<(), OurError> {
        let file = axum2::File {
            id: axum2::gen_id().unwrap(),
            file_name: String::from("abc.txt"),
            file_type: file_type.into(),
            size: 12,
            sha256: Some(String::from(
                "c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a",
            )),
            preview: Some(String::from("Hello world!")),
        };
        let mut files = HashMap::new();
        files.insert(file.id.clone(), file);
        let json = axum2::PrepareUploadRequest {
            info: self.config.info.clone(),
            files,
        };
        let client = reqwest::Client::new();
        let res = client
            .post("http://192.168.42.184:53317/api/localsend/v2/prepare-upload")
            .json(&json)
            .send()
            .await?;
        match res.status() {
            reqwest::StatusCode::OK => {
                println!("{res:?}");
                let response: axum2::PrepareUploadResponse = res.json().await?;
                println!("{response:?}");
                for (file_id, token) in response.files {
                    let session_id = response.session_id.clone();
                    let res = client
                        .post("http://192.168.42.184:53317/api/localsend/v2/upload")
                        .query(&[
                            ("sessionId", session_id),
                            ("fileId", file_id),
                            ("token", token),
                        ])
                        .body(data.clone())
                        .send()
                        .await?;
                    println!("{res:?}");
                    let json = res.text().await?;
                    println!("{json:?}");
                }
            }
            reqwest::StatusCode::NO_CONTENT => (),
            reqwest::StatusCode::FORBIDDEN => todo!("403"),
            status => todo!("{status:?}"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
