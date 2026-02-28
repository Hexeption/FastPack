//! Self-update logic. Checks GitHub releases, downloads platform-specific
//! installers, and applies them.

use std::path::PathBuf;

/// Crate version compiled into the binary.
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// GitHub repository used for release checks.
const REPO: &str = "Hexeption/FastPack";

/// Describes a newer release found on GitHub.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReleaseInfo {
    /// Tag name from the GitHub release (e.g. `"v0.28.0"`).
    pub version: String,
    /// Release notes body (Markdown).
    pub notes: String,
    /// Direct download URL for the platform-specific asset.
    pub asset_url: String,
}

/// Fetch the latest release from GitHub. Returns `None` if no matching asset exists.
fn check_latest() -> Result<Option<ReleaseInfo>, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let resp = ureq::get(&url)
        .header("User-Agent", &format!("fastpack/{CURRENT_VERSION}"))
        .call()
        .map_err(|e| e.to_string())?;

    let body_str = resp
        .into_body()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| e.to_string())?;

    let version = json["tag_name"].as_str().unwrap_or("").to_string();
    let notes = json["body"].as_str().unwrap_or("").to_string();

    let asset_suffix = platform_asset_suffix();
    let asset_url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|a| {
                a["name"]
                    .as_str()
                    .is_some_and(|n| n.ends_with(asset_suffix))
            })
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .map(|s| s.to_string());

    let Some(asset_url) = asset_url else {
        return Ok(None);
    };

    Ok(Some(ReleaseInfo {
        version,
        notes,
        asset_url,
    }))
}

/// Check if a newer version is available. Returns `Some(release)` when the
/// latest tag is strictly newer than `CURRENT_VERSION`.
pub fn do_check() -> Result<Option<ReleaseInfo>, String> {
    let info = check_latest()?;
    match info {
        None => Ok(None),
        Some(release) => {
            let current = parse_version(CURRENT_VERSION);
            let latest = parse_version(&release.version);
            if latest > current {
                Ok(Some(release))
            } else {
                Ok(None)
            }
        }
    }
}

/// Download the release asset to a temp file. Returns the path on disk.
pub fn do_download(url: &str) -> Result<PathBuf, String> {
    let resp = ureq::get(url)
        .header("User-Agent", &format!("fastpack/{CURRENT_VERSION}"))
        .call()
        .map_err(|e| e.to_string())?;

    let dest = std::env::temp_dir().join(download_filename());
    let mut file = std::fs::File::create(&dest).map_err(|e| e.to_string())?;
    let mut reader = resp.into_body().into_reader();
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    Ok(dest)
}

/// Parse a semver-like string into `(major, minor, patch)`.
fn parse_version(v: &str) -> (u32, u32, u32) {
    let v = v.trim_start_matches('v');
    let mut parts = v.split('.').filter_map(|p| p.parse::<u32>().ok());
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

/// Return the expected asset filename suffix for this OS and architecture.
#[cfg(target_os = "windows")]
fn platform_asset_suffix() -> &'static str {
    "windows-x86_64.msi"
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn platform_asset_suffix() -> &'static str {
    "macos-aarch64.dmg"
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn platform_asset_suffix() -> &'static str {
    "macos-x86_64.dmg"
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_asset_suffix() -> &'static str {
    "linux-x86_64.tar.gz"
}

#[cfg(target_os = "windows")]
fn download_filename() -> &'static str {
    "fastpack_update.msi"
}

#[cfg(target_os = "macos")]
fn download_filename() -> &'static str {
    "fastpack_update.dmg"
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn download_filename() -> &'static str {
    "fastpack_update.tar.gz"
}

/// Apply a downloaded update. Opens the installer or replaces the binary in place.
pub fn do_apply(downloaded: &std::path::Path) -> Result<(), String> {
    apply_impl(downloaded)
}

#[cfg(target_os = "windows")]
fn apply_impl(downloaded: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("msiexec")
        .args(["/i", downloaded.to_str().unwrap_or(""), "/passive"])
        .spawn()
        .map_err(|e| e.to_string())?;
    std::process::exit(0);
}

#[cfg(target_os = "macos")]
fn apply_impl(downloaded: &std::path::Path) -> Result<(), String> {
    std::process::Command::new("open")
        .arg(downloaded)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn apply_impl(downloaded: &std::path::Path) -> Result<(), String> {
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    let extract_dir = std::env::temp_dir().join("fastpack_update_extract");
    std::fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;

    let status = std::process::Command::new("tar")
        .args([
            "-xzf",
            downloaded.to_str().unwrap_or(""),
            "-C",
            extract_dir.to_str().unwrap_or(""),
        ])
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("failed to extract update archive".to_string());
    }

    let extracted = extract_dir.join("fastpack");
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&extracted)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&extracted, perms).map_err(|e| e.to_string())?;
    }
    std::fs::copy(&extracted, &current).map_err(|e| e.to_string())?;
    std::process::Command::new(&current)
        .spawn()
        .map_err(|e| e.to_string())?;
    std::process::exit(0);
}
