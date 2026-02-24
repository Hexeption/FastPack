use thiserror::Error;

/// Errors produced by image encoding and compression operations.
#[derive(Debug, Error)]
pub enum CompressError {
    #[error("image encoding failed: {0}")]
    ImageEncode(#[from] image::ImageError),
    #[cfg(feature = "png")]
    #[error("png optimization failed: {0}")]
    /// PNG optimization via oxipng failed.
    PngOptimize(#[from] oxipng::PngError),
    #[error("{0}")]
    /// An error not covered by the other variants.
    Other(String),
}
