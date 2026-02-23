use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompressError {
    #[error("image encoding failed: {0}")]
    ImageEncode(#[from] image::ImageError),
    #[cfg(feature = "png")]
    #[error("png optimization failed: {0}")]
    PngOptimize(#[from] oxipng::PngError),
    #[error("{0}")]
    Other(String),
}
