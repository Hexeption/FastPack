use thiserror::Error;

/// Errors produced by image encoding and compression operations.
#[derive(Debug, Error)]
pub enum CompressError {
    #[error("image encoding failed: {0}")]
    ImageEncode(#[from] image::ImageError),
    /// PNG optimization via oxipng failed.
    #[cfg(feature = "png")]
    #[error("png optimization failed: {0}")]
    PngOptimize(#[from] oxipng::PngError),
    /// An error not covered by the other variants.
    #[error("{0}")]
    Other(String),
}
