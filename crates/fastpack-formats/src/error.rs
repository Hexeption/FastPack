use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("json serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
