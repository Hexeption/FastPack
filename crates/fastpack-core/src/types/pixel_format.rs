use serde::{Deserialize, Serialize};

/// Pixel-level encoding of atlas data (before container compression).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PixelFormat {
    #[default]
    Rgba8888,
    Rgb888,
    Rgb565,
    Rgba4444,
    Rgba5551,
    Alpha8,
}

impl std::fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rgba8888 => write!(f, "RGBA8888"),
            Self::Rgb888 => write!(f, "RGB888"),
            Self::Rgb565 => write!(f, "RGB565"),
            Self::Rgba4444 => write!(f, "RGBA4444"),
            Self::Rgba5551 => write!(f, "RGBA5551"),
            Self::Alpha8 => write!(f, "Alpha8"),
        }
    }
}

impl std::str::FromStr for PixelFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rgba8888" => Ok(Self::Rgba8888),
            "rgb888" => Ok(Self::Rgb888),
            "rgb565" => Ok(Self::Rgb565),
            "rgba4444" => Ok(Self::Rgba4444),
            "rgba5551" => Ok(Self::Rgba5551),
            "alpha8" => Ok(Self::Alpha8),
            _ => Err(format!("unknown pixel format: {s}")),
        }
    }
}

/// Output container / hardware texture format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextureFormat {
    #[default]
    Png,
    Jpeg,
    WebP,
    Etc1,
    Etc2,
    Pvrtc1,
    Pvrtc2,
    Dxt1,
    Dxt5,
    Astc,
    Basis,
}

impl std::fmt::Display for TextureFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Png => write!(f, "png"),
            Self::Jpeg => write!(f, "jpeg"),
            Self::WebP => write!(f, "webp"),
            Self::Etc1 => write!(f, "etc1"),
            Self::Etc2 => write!(f, "etc2"),
            Self::Pvrtc1 => write!(f, "pvrtc1"),
            Self::Pvrtc2 => write!(f, "pvrtc2"),
            Self::Dxt1 => write!(f, "dxt1"),
            Self::Dxt5 => write!(f, "dxt5"),
            Self::Astc => write!(f, "astc"),
            Self::Basis => write!(f, "basis"),
        }
    }
}

impl std::str::FromStr for TextureFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpeg" | "jpg" => Ok(Self::Jpeg),
            "webp" => Ok(Self::WebP),
            "etc1" => Ok(Self::Etc1),
            "etc2" => Ok(Self::Etc2),
            "pvrtc1" => Ok(Self::Pvrtc1),
            "pvrtc2" => Ok(Self::Pvrtc2),
            "dxt1" | "bc1" => Ok(Self::Dxt1),
            "dxt5" | "bc3" => Ok(Self::Dxt5),
            "astc" => Ok(Self::Astc),
            "basis" => Ok(Self::Basis),
            _ => Err(format!("unknown texture format: {s}")),
        }
    }
}

impl TextureFormat {
    /// File extension (without leading dot) for the output texture file.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Etc1 | Self::Etc2 => "ktx",
            Self::Pvrtc1 | Self::Pvrtc2 => "pvr",
            Self::Dxt1 | Self::Dxt5 => "dds",
            Self::Astc => "astc",
            Self::Basis => "basis",
        }
    }
}
