use thiserror::Error;

/// Errors produced by data format export writers.
#[derive(Debug, Error)]
pub enum FormatError {
    #[error("json serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    /// An error not covered by the other variants.
    #[error("{0}")]
    Other(String),
}
