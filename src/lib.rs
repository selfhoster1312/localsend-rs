use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use tokio::net::{TcpListener, UdpSocket};

pub mod axum2;
pub mod info;
pub mod random;

use info::{DeviceType, Info, Protocol};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Announce {
    #[serde(flatten)]
    info: Info,
    announce: bool,
}

#[derive(Debug)]
pub enum OurError {
    Json(serde_json::Error),
    Io(io::Error),
    Reqwest(reqwest::Error),
    NoXDG,
}

impl From<serde_json::Error> for OurError {
    fn from(err: serde_json::Error) -> OurError {
        OurError::Json(err)
    }
}

impl From<io::Error> for OurError {
    fn from(err: io::Error) -> OurError {
        OurError::Io(err)
    }
}

impl From<reqwest::Error> for OurError {
    fn from(err: reqwest::Error) -> OurError {
        OurError::Reqwest(err)
    }
}

pub struct LocalSend {
    /// Information about the current device/session
    info: Info,
    udp_socket: UdpSocket,
}

impl LocalSend {
    pub async fn new(info: Info) -> Result<LocalSend, OurError> {
        let info2 = info.clone();
        tokio::task::spawn(async {
            // TODO: configure port from info.port
            let tcp_listener = TcpListener::bind("0.0.0.0:53317").await.unwrap();
            let app = crate::axum2::route(info2);
            axum::serve(tcp_listener, app).await.unwrap();
        });

        // TODO: Add support for IPv6.
        let udp_socket = UdpSocket::bind("224.0.0.167:53317").await?;
        Ok(LocalSend { info, udp_socket })
    }

    pub async fn from_xdg() -> Result<LocalSend, OurError> {
        Self::new(Info::from_xdg().await?).await
    }

    pub async fn send_announce(&self) -> Result<(), OurError> {
        let announce = Announce {
            announce: true,
            info: self.info.clone(),
        };
        let json = serde_json::to_string(&announce)?;
        println!("{json}");

        self.udp_socket
            .send_to(json.as_bytes(), "224.0.0.167:53317")
            .await?;

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
        let json = axum2::PrepareUploadRequest { info: self.info.clone(), files };
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
