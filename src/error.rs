#[derive(Debug)]
pub enum OurError {
    Json(serde_json::Error),
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    NoXDG,
    Tokio(tokio::task::JoinError),
}

impl std::error::Error for OurError {}

impl std::fmt::Display for OurError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Json(e) => format!("JSON error: {e}"),
                Self::Io(e) => format!("IO error: {e}"),
                Self::Reqwest(e) => format!("Client HTTP error: {e}"),
                Self::NoXDG => format!("Could not find $XDG_CONFIG_DIR or $HOME."),
                Self::Tokio(e) => format!("Execution error: {e}"),
            }
        )
    }
}

impl From<serde_json::Error> for OurError {
    fn from(err: serde_json::Error) -> OurError {
        OurError::Json(err)
    }
}

impl From<std::io::Error> for OurError {
    fn from(err: std::io::Error) -> OurError {
        OurError::Io(err)
    }
}

impl From<reqwest::Error> for OurError {
    fn from(err: reqwest::Error) -> OurError {
        OurError::Reqwest(err)
    }
}

impl From<tokio::task::JoinError> for OurError {
    fn from(err: tokio::task::JoinError) -> OurError {
        OurError::Tokio(err)
    }
}
