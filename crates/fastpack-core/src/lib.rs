//! Core sprite packing library for FastPack.
//!
//! Handles image loading, trimming, extrusion, alias detection, and rectangle
//! bin-packing. All engine-agnostic logic lives here. The CLI and GUI frontends
//! depend on this crate but never the reverse.

#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

pub mod algorithms;
pub mod error;
pub mod imaging;
pub mod multipack;
pub mod types;
