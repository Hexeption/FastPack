//! Image loading and per-sprite processing pipeline.
//!
//! Covers loading from disk, transparent-border trimming, border extrusion,
//! alpha bleeding, Floyd-Steinberg dithering, premultiplied alpha, pixel-art upscaling,
//! alias detection, and scale-variant generation.

pub mod alias;
pub mod bleed;
pub mod dither;
pub mod extrude;
pub mod loader;
pub mod naming;
pub mod ninepatch;
pub mod pivot;
pub mod pixel_art;
pub mod premultiply;
pub mod scale;
pub mod trim;
