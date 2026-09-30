//! Sprite id and export-name derivation.
//!
//! A sprite has two names:
//!
//! - its **id**: the forward-slash path relative to its source root with the
//!   file extension removed (e.g. `"ui/button"`). The id does not depend on any
//!   naming option and is the key used by `sprite_overrides`, `excludes`, and
//!   the GUI selection.
//! - its **export name**: the frame name written to data files. It starts from
//!   the relative path and is adjusted by [`SpriteConfig::keep_extension`] and
//!   [`SpriteConfig::prepend_folder_name`].

use std::path::Path;

use crate::types::config::SpriteConfig;

/// Derive the internal sprite id for `path` relative to `base`.
///
/// Returns the relative path without its extension, using `/` as separator.
pub fn sprite_id(path: &Path, base: &Path) -> String {
    let rel = path.strip_prefix(base).unwrap_or(path);
    normalize(&rel.with_extension(""))
}

/// Derive the export (frame) name for `path` relative to `base`.
///
/// `source_dir` is the source directory the sprite was found in, or `None` when
/// the source was a single file. When [`SpriteConfig::prepend_folder_name`] is
/// set and a source directory is given, that directory's own name is prepended
/// (e.g. source `assets/hero` yields `hero/run_01`).
pub fn sprite_name(
    path: &Path,
    base: &Path,
    source_dir: Option<&Path>,
    cfg: &SpriteConfig,
) -> String {
    let rel = path.strip_prefix(base).unwrap_or(path);
    let name = if cfg.keep_extension {
        normalize(rel)
    } else {
        normalize(&rel.with_extension(""))
    };
    match source_dir
        .filter(|_| cfg.prepend_folder_name)
        .and_then(folder_name)
    {
        Some(folder) => format!("{folder}/{name}"),
        None => name,
    }
}

fn normalize(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// The last component of `dir`, resolving `.`/`..` via canonicalization when needed.
fn folder_name(dir: &Path) -> Option<String> {
    let last = |p: &Path| {
        p.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|n| !n.is_empty())
    };
    last(dir).or_else(|| dir.canonicalize().ok().as_deref().and_then(last))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(keep_extension: bool, prepend_folder_name: bool) -> SpriteConfig {
        SpriteConfig {
            keep_extension,
            prepend_folder_name,
            ..SpriteConfig::default()
        }
    }

    #[test]
    fn id_strips_extension() {
        let base = Path::new("assets/hero");
        let path = Path::new("assets/hero/run/run_01.png");
        assert_eq!(sprite_id(path, base), "run/run_01");
    }

    #[test]
    fn name_defaults_match_id() {
        let base = Path::new("assets/hero");
        let path = Path::new("assets/hero/run/run_01.png");
        assert_eq!(
            sprite_name(path, base, Some(base), &cfg(false, false)),
            "run/run_01"
        );
    }

    #[test]
    fn name_keeps_extension() {
        let base = Path::new("assets/hero");
        let path = Path::new("assets/hero/run_01.png");
        assert_eq!(
            sprite_name(path, base, Some(base), &cfg(true, false)),
            "run_01.png"
        );
    }

    #[test]
    fn name_prepends_folder() {
        let base = Path::new("assets/hero/");
        let path = Path::new("assets/hero/run_01.png");
        assert_eq!(
            sprite_name(path, base, Some(base), &cfg(false, true)),
            "hero/run_01"
        );
        assert_eq!(
            sprite_name(path, base, Some(base), &cfg(true, true)),
            "hero/run_01.png"
        );
    }

    #[test]
    fn single_file_source_gets_no_folder_prefix() {
        let base = Path::new("assets/hero");
        let path = Path::new("assets/hero/run_01.png");
        assert_eq!(
            sprite_name(path, base, None, &cfg(true, true)),
            "run_01.png"
        );
    }
}
