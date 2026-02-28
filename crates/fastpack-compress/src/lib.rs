//! Image compression backends for atlas textures (PNG, JPEG, WebP, DXT).
//!
//! Each backend implements the [`compressor::Compressor`] trait: encode a
//! composited atlas [`DynamicImage`](image::DynamicImage) into output bytes.
//! The pipeline selects a backend by [`TextureFormat`](fastpack_core::types::pixel_format::TextureFormat)
//! and calls `compress()`.

#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

pub mod backends;
pub mod compressor;
pub mod error;
