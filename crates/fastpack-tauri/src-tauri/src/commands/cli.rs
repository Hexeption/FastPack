use std::path::PathBuf;

const SYMLINK_PATH: &str = "/usr/local/bin/fastpack";

fn cli_path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("failed to get exe path: {e}"))?;
    let dir = exe.parent().ok_or("failed to get exe directory")?;
    let cli = dir.join("fastpack");
    if cli.exists() {
        Ok(cli)
    } else {
        Err(format!("CLI binary not found at {}", cli.display()))
    }
}

#[cfg(unix)]
fn do_install(exe: &PathBuf) -> Result<(), String> {
    let target = PathBuf::from(SYMLINK_PATH);
    let exe_str = exe.display();

    // Try direct symlink first, fall back to privilege escalation
    if try_direct_symlink(exe, &target).is_ok() {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "do shell script \"rm -f '{SYMLINK_PATH}' && ln -s '{exe_str}' '{SYMLINK_PATH}'\" with administrator privileges"
        );
        let status = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .status()
            .map_err(|e| format!("failed to run osascript: {e}"))?;
        if !status.success() {
            return Err("privilege escalation cancelled or failed".into());
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Linux: try pkexec
        let status = std::process::Command::new("pkexec")
            .arg("bash")
            .arg("-c")
            .arg(format!(
                "rm -f '{SYMLINK_PATH}' && ln -s '{exe_str}' '{SYMLINK_PATH}'"
            ))
            .status()
            .map_err(|e| format!("failed to run pkexec: {e}"))?;
        if !status.success() {
            return Err("privilege escalation cancelled or failed".into());
        }
        Ok(())
    }
}

#[cfg(unix)]
fn try_direct_symlink(exe: &PathBuf, target: &PathBuf) -> Result<(), ()> {
    let parent = target.parent().unwrap();
    if !parent.exists() {
        std::fs::create_dir_all(parent).map_err(|_| ())?;
    }
    if target.exists() || target.symlink_metadata().is_ok() {
        std::fs::remove_file(target).map_err(|_| ())?;
    }
    std::os::unix::fs::symlink(exe, target).map_err(|_| ())
}

#[cfg(unix)]
fn do_check(exe: &PathBuf) -> bool {
    let target = PathBuf::from(SYMLINK_PATH);
    match std::fs::read_link(&target) {
        Ok(dest) => dest == *exe,
        Err(_) => false,
    }
}

#[cfg(windows)]
fn do_install(exe: &PathBuf) -> Result<(), String> {
    let exe_dir = exe
        .parent()
        .ok_or("failed to get exe directory")?
        .to_string_lossy()
        .to_string();

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let (env, _) = hkcu
        .create_subkey("Environment")
        .map_err(|e| format!("failed to open registry: {e}"))?;

    let current: String = env.get_value("Path").unwrap_or_default();
    if current.split(';').any(|p| p.eq_ignore_ascii_case(&exe_dir)) {
        return Ok(());
    }

    let new_path = if current.is_empty() {
        exe_dir
    } else {
        format!("{current};{exe_dir}")
    };

    env.set_value("Path", &new_path)
        .map_err(|e| format!("failed to set PATH: {e}"))?;

    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::SendMessageTimeoutW(
            windows_sys::Win32::UI::WindowsAndMessaging::HWND_BROADCAST,
            windows_sys::Win32::UI::WindowsAndMessaging::WM_SETTINGCHANGE,
            0,
            "Environment\0"
                .encode_utf16()
                .collect::<Vec<u16>>()
                .as_ptr() as isize,
            windows_sys::Win32::UI::WindowsAndMessaging::SMTO_ABORTIFHUNG,
            5000,
            std::ptr::null_mut(),
        );
    }

    Ok(())
}

#[cfg(windows)]
fn do_check(exe: &PathBuf) -> bool {
    let exe_dir = match exe.parent() {
        Some(p) => p.to_string_lossy().to_string(),
        None => return false,
    };

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let env = match hkcu.open_subkey("Environment") {
        Ok(k) => k,
        Err(_) => return false,
    };

    let current: String = env.get_value("Path").unwrap_or_default();
    current.split(';').any(|p| p.eq_ignore_ascii_case(&exe_dir))
}

#[tauri::command]
pub fn install_cli() -> Result<(), String> {
    let cli = cli_path()?;
    do_install(&cli)
}

#[tauri::command]
pub fn check_cli_installed() -> Result<bool, String> {
    let cli = cli_path()?;
    Ok(do_check(&cli))
}
