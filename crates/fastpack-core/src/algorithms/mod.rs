//! Packing algorithms that place sprites onto atlas sheets.
//!
//! Each algorithm implements the [`packer::Packer`] trait. Available strategies
//! range from simple row-strip and grid placement to the MaxRects bin-packer.

pub mod basic;
pub mod grid;
pub mod maxrects;
pub mod packer;
pub mod polygon;
