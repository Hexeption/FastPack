use thiserror::Error;

/// Errors produced by core image loading and packing operations.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("failed to load image '{path}': {source}")]
    ImageLoad {
        path: std::path::PathBuf,
        #[source]
        /// The underlying image loading error.
        source: image::ImageError,
    },

    /// Sprite dimensions exceed the configured maximum atlas size.
    #[error("sprite '{id}' ({w}×{h} px) is too large to fit in a {max_w}×{max_h} atlas")]
    SpriteTooLarge {
        /// Unique identifier of the oversized sprite.
        id: String,
        /// Sprite width in pixels.
        w: u32,
        /// Sprite height in pixels.
        h: u32,
        /// Maximum atlas width in pixels.
        max_w: u32,
        /// Maximum atlas height in pixels.
        max_h: u32,
    },

    /// All configured source directories contained no sprites.
    #[error("cannot pack an empty sprite list")]
    NoSprites,

    /// The requested export format string does not match any known exporter.
    #[error("unknown export format '{0}'")]
    UnknownFormat(String),

    /// A filesystem I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
