use crate::{
    error::FormatError,
    exporter::{ExportInput, Exporter, SpriteRotation},
    formats::libgdx::{AtlasStyle, write_atlas},
};

/// Exports atlas metadata in the Spine 4 `.atlas` text format.
///
/// Spine's format is the compact variant of the libGDX atlas (`bounds:`,
/// `offsets:`, `rotate:90`, `split:`). Region names are sprite ids without a
/// file extension, matching Spine attachment paths. Rotated regions are stored
/// 90° counter-clockwise. Multipack sheets are combined into one file with one
/// page per texture.
pub struct SpineExporter;

const SPINE: AtlasStyle = AtlasStyle {
    key_sep: ":",
    value_sep: ",",
    rotate_value: "90",
    name: "Spine",
};

impl Exporter for SpineExporter {
    fn export(&self, input: &ExportInput<'_>) -> Result<String, FormatError> {
        write_atlas(std::slice::from_ref(input), &SPINE)
    }

    fn combine(&self, inputs: &[ExportInput<'_>]) -> Option<Result<String, FormatError>> {
        Some(write_atlas(inputs, &SPINE))
    }

    fn rotation(&self) -> SpriteRotation {
        SpriteRotation::CounterClockwise
    }

    fn format_id(&self) -> &'static str {
        "spine"
    }

    fn file_extension(&self) -> &'static str {
        "atlas"
    }
}
