use std::path::PathBuf;
use std::sync::mpsc;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "Hexeption/FastPack";

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    /// Release notes text from GitHub.
    pub notes: String,
    /// Direct download URL for the platform-specific binary.
    pub asset_url: String,
}

/// Messages sent from the update background thread to the UI.
pub enum UpdateMsg {
    /// Current version is the latest.
    UpToDate { latest: String },
    /// A newer release is available.
    Available(ReleaseInfo),
    /// The update binary was downloaded to the given path.
    Downloaded(PathBuf),
    /// The update check or download failed.
    Error(String),
}

/// Current state of the update check shown in the preferences window.
pub enum UpdateStatus {
    /// No check has been started.
    Idle,
    /// A check is running in the background.
    Checking,
    /// Check completed; running version is the latest.
    UpToDate { latest: String },
    /// A newer release was found.
    Available(ReleaseInfo),
    /// The update binary is being downloaded.
    Downloading,
    /// Download finished; binary is at the given path.
    Downloaded(PathBuf),
    /// The check or download failed with this message.
    Error(String),
}

/// Spawn a background thread to check GitHub for the latest release.
pub fn spawn_check(tx: mpsc::Sender<UpdateMsg>) {
    std::thread::spawn(move || {
        let msg = match check_latest() {
            Err(e) => UpdateMsg::Error(e),
            Ok(None) => UpdateMsg::UpToDate {
                latest: CURRENT_VERSION.to_string(),
            },
            Ok(Some(release)) => {
                let current = parse_version(CURRENT_VERSION);
                let latest = parse_version(&release.version);
                if latest > current {
                    UpdateMsg::Available(release)
                } else {
                    UpdateMsg::UpToDate {
                        latest: release.version,
                    }
                }
            }
        };
        tx.send(msg).ok();
    });
}

/// Spawn a background thread to download the release asset.
pub fn spawn_download(release: ReleaseInfo, tx: mpsc::Sender<UpdateMsg>) {
    std::thread::spawn(move || {
        let msg = match download_asset(&release.asset_url) {
            Ok(path) => UpdateMsg::Downloaded(path),
            Err(e) => UpdateMsg::Error(e),
        };
        tx.send(msg).ok();
    });
}

/// Replace the running binary with the downloaded update file and restart.
pub fn apply_update(downloaded: &std::path::Path) -> Result<(), String> {
    do_apply(downloaded)
}

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

fn download_asset(url: &str) -> Result<PathBuf, String> {
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

fn parse_version(v: &str) -> (u32, u32, u32) {
    let v = v.trim_start_matches('v');
    let mut parts = v.split('.').filter_map(|p| p.parse::<u32>().ok());
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

#[cfg(target_os = "windows")]
fn platform_asset_suffix() -> &'static str {
    "windows-x86_64.exe"
}

#[cfg(target_os = "macos")]
fn platform_asset_suffix() -> &'static str {
    "macos-aarch64"
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_asset_suffix() -> &'static str {
    "linux-x86_64"
}

#[cfg(target_os = "windows")]
fn download_filename() -> &'static str {
    "fastpack_update_download.exe"
}

#[cfg(not(target_os = "windows"))]
fn download_filename() -> &'static str {
    "fastpack_update_download"
}

#[cfg(target_os = "windows")]
fn do_apply(downloaded: &std::path::Path) -> Result<(), String> {
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    let bat = format!(
        "@echo off\r\ntimeout /t 2 /nobreak >nul\r\ncopy /y \"{src}\" \"{dst}\"\r\nstart \"\" \"{dst}\"\r\n",
        src = downloaded.display(),
        dst = current.display(),
    );
    let bat_path = std::env::temp_dir().join("fastpack_update.bat");
    std::fs::write(&bat_path, bat.as_bytes()).map_err(|e| e.to_string())?;
    std::process::Command::new("cmd")
        .args(["/C", bat_path.to_str().unwrap_or("")])
        .spawn()
        .map_err(|e| e.to_string())?;
    std::process::exit(0);
}

#[cfg(not(target_os = "windows"))]
fn do_apply(downloaded: &std::path::Path) -> Result<(), String> {
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(downloaded)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(downloaded, perms).map_err(|e| e.to_string())?;
    }
    std::fs::copy(downloaded, &current).map_err(|e| e.to_string())?;
    std::process::Command::new(&current)
        .spawn()
        .map_err(|e| e.to_string())?;
    std::process::exit(0);
}
