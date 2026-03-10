use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Engine not implemented: {0}")]
    NotImplemented(String),

    #[error("Invalid engine: {0}")]
    InvalidEngine(String),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("YouTube download error: {0}")]
    YoutubeDownload(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
