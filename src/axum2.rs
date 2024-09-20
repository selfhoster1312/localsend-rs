use crate::info::{DeviceType, Info, Protocol, SavedConfig};
use axum::{
    body::Bytes,
    extract::{Json, Query, State},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct File {
    pub id: String,
    pub file_name: String,
    pub file_type: String,
    pub size: usize,
    pub sha256: Option<String>,
    pub preview: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareUploadRequest {
    pub info: Info,
    pub files: HashMap<String, File>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareUploadResponse {
    pub session_id: String,
    pub files: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UploadQuery {
    session_id: String,
    file_id: String,
    token: String,
}

async fn post_register() -> Json<Info> {
    println!("Register!");
    axum::Json(Info {
        config: SavedConfig::new_random(),
        version: String::from("2.0"),
        device_model: Some(String::from("Linux")),
        device_type: Some(DeviceType::Desktop),
        port: 53317,
        protocol: Protocol::Http,
        download: true,
    })
}

pub fn gen_id() -> Result<String, getrandom::Error> {
    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf)?;
    let mut string = String::with_capacity(16);
    for byte in buf {
        string.extend(format!("{byte:02x}").chars());
    }
    Ok(string)
}

async fn post_prepare_upload(
    State(state): State<Arc<OurState>>,
    Json(payload): Json<PrepareUploadRequest>,
) -> Json<PrepareUploadResponse> {
    println!("Prepare upload!");
    println!("{payload:?}");
    println!("{state:?}");
    let mut files = HashMap::new();
    for (id, file) in payload.files.into_iter() {
        println!("Do you want {file:?}? [Y/n]");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();
        if buf == "Y" || buf == "y" || buf == "" {
            files.insert(id, gen_id().unwrap());
        }
    }
    Json(PrepareUploadResponse {
        session_id: gen_id().unwrap(),
        files,
    })
}

async fn post_upload(Query(params): Query<UploadQuery>, body: Bytes) {
    println!("Upload!");
    println!("{params:?}");
    println!("{body:?}");
}

async fn post_prepare_download() {
    println!("Prepare download!");
}

async fn get_download() {
    println!("Download!");
}

async fn post_cancel() {
    println!("Cancel!");
}

#[derive(Debug)]
struct OurState {}

impl OurState {
    fn new() -> OurState {
        OurState {}
    }
}

pub fn route() -> Router {
    let state = Arc::new(OurState::new());
    Router::new()
        .route("/api/localsend/v2/register", post(post_register))
        .route(
            "/api/localsend/v2/prepare-upload",
            post(post_prepare_upload),
        )
        .route("/api/localsend/v2/upload", post(post_upload))
        .route(
            "/api/localsend/v2/prepare-download",
            post(post_prepare_download),
        )
        .route("/api/localsend/v2/download", get(get_download))
        .route("/api/localsend/v2/cancel", post(post_cancel))
        .with_state(state)

    // Legacy endpoint, not used.
    //.route("/api/localsend/v2/info", get(get_info))
}
