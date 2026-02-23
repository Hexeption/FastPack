use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("failed to load image '{path}': {source}")]
    ImageLoad {
        path: std::path::PathBuf,
        #[source]
        source: image::ImageError,
    },

    #[error("sprite '{id}' ({w}×{h} px) is too large to fit in a {max_w}×{max_h} atlas")]
    SpriteTooLarge {
        id: String,
        w: u32,
        h: u32,
        max_w: u32,
        max_h: u32,
    },

    #[error("cannot pack an empty sprite list")]
    NoSprites,

    #[error("unknown export format '{0}'")]
    UnknownFormat(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
