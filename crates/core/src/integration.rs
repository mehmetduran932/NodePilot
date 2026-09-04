//! Shell and PATH integration management with safe rollback support.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::error::NodePilotError;
use crate::paths::NodePilotPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationBackup {
    pub previous_path: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationStatus {
    pub is_enabled: bool,
    pub bin_dir: PathBuf,
    pub in_current_process_path: bool,
    pub in_persisted_user_path: bool,
    pub backup_available: bool,
}

/// Checks the current status of NodePilot shell integration.
pub fn get_integration_status(paths: &NodePilotPaths) -> IntegrationStatus {
    let bin_dir = paths.bin_dir.clone();
    let current_path = std::env::var("PATH").unwrap_or_default();
    let in_current = current_path.contains(&bin_dir.to_string_lossy().to_string());

    let in_persisted = check_persisted_path(&bin_dir);
    let backup_file = paths.config_dir.join("integration_backup.json");
    let backup_available = backup_file.is_file();

    IntegrationStatus {
        is_enabled: in_persisted,
        bin_dir,
        in_current_process_path: in_current,
        in_persisted_user_path: in_persisted,
        backup_available,
    }
}

/// Enables NodePilot shell integration by adding bin_dir to the user's PATH.
/// Backs up previous user PATH for rollback.
pub fn enable_integration(paths: &NodePilotPaths) -> Result<(), NodePilotError> {
    let bin_str = paths.bin_dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_EXPAND_SZ};
        use winreg::{RegKey, RegValue};

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| NodePilotError::Integration(format!("Cannot open registry Environment: {}", e)))?;

        let current_user_path: String = env.get_value("Path").unwrap_or_default();

        // Check if already in PATH
        let entries: Vec<&str> = current_user_path.split(';').map(|s| s.trim()).collect();
        if entries.iter().any(|&e| e.eq_ignore_ascii_case(&bin_str)) {
            return Ok(()); // Already present
        }

        // Backup existing PATH
        let backup = IntegrationBackup {
            previous_path: current_user_path.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        let backup_file = paths.config_dir.join("integration_backup.json");
        if let Some(parent) = backup_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&backup_file, serde_json::to_string_pretty(&backup)?);

        // Prepend NodePilot bin to ensure high priority routing
        let new_path = if current_user_path.is_empty() {
            bin_str.clone()
        } else {
            format!("{};{}", bin_str, current_user_path)
        };

        // Preserve REG_EXPAND_SZ so %USERPROFILE% and other variables expand properly
        let reg_val = RegValue {
            vtype: REG_EXPAND_SZ,
            bytes: new_path.encode_utf16().chain(std::iter::once(0)).flat_map(|u| u.to_le_bytes()).collect(),
        };

        env.set_raw_value("Path", &reg_val)
            .map_err(|e| NodePilotError::Integration(format!("Failed to write user PATH registry: {}", e)))?;

        // Broadcast WM_SETTINGCHANGE so new shells pick it up
        broadcast_env_change();
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix/macOS, record in dedicated profile script or ~/.nodepilot/env
        let env_file = paths.root.join("nodepilot.env");
        let content = format!("export PATH=\"{}:$PATH\"\n", bin_str);
        std::fs::write(&env_file, content)?;
    }

    Ok(())
}

/// Disables NodePilot shell integration and cleanly removes bin_dir from user PATH.
pub fn disable_integration(paths: &NodePilotPaths) -> Result<(), NodePilotError> {
    #[cfg(target_os = "windows")]
    let bin_str = paths.bin_dir.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_EXPAND_SZ};
        use winreg::{RegKey, RegValue};

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| NodePilotError::Integration(format!("Cannot open registry Environment: {}", e)))?;

        let current_user_path: String = env.get_value("Path").unwrap_or_default();
        let entries: Vec<&str> = current_user_path.split(';')
            .map(|s| s.trim())
            .filter(|&e| !e.is_empty() && !e.eq_ignore_ascii_case(&bin_str))
            .collect();

        let new_path = entries.join(";");
        let reg_val = RegValue {
            vtype: REG_EXPAND_SZ,
            bytes: new_path.encode_utf16().chain(std::iter::once(0)).flat_map(|u| u.to_le_bytes()).collect(),
        };

        env.set_raw_value("Path", &reg_val)
            .map_err(|e| NodePilotError::Integration(format!("Failed to update user PATH registry: {}", e)))?;

        broadcast_env_change();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let env_file = paths.root.join("nodepilot.env");
        if env_file.exists() {
            let _ = std::fs::remove_file(env_file);
        }
    }

    Ok(())
}

fn check_persisted_path(bin_dir: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(env) = hkcu.open_subkey_with_flags("Environment", KEY_READ) {
            let user_path: String = env.get_value("Path").unwrap_or_default();
            let bin_str = bin_dir.to_string_lossy().to_string();
            return user_path.split(';').any(|s| s.trim().eq_ignore_ascii_case(&bin_str));
        }
        false
    }

    #[cfg(not(target_os = "windows"))]
    {
        let env_file = bin_dir.parent().map(|p| p.join("nodepilot.env"));
        env_file.map(|f| f.exists()).unwrap_or(false)
    }
}

#[cfg(target_os = "windows")]
fn broadcast_env_change() {
    // WM_SETTINGCHANGE broadcast via SendMessageTimeoutW
    // Safety: Windows standard API call
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn SendMessageTimeoutW(
                hwnd: *mut std::ffi::c_void,
                msg: u32,
                wparam: usize,
                lparam: *const u16,
                flags: u32,
                timeout: u32,
                result: *mut usize,
            ) -> isize;
        }

        const HWND_BROADCAST: *mut std::ffi::c_void = 0xffff as *mut std::ffi::c_void;
        const WM_SETTINGCHANGE: u32 = 0x001A;
        const SMTO_ABORTIFHUNG: u32 = 0x0002;

        let env_str: Vec<u16> = "Environment\0".encode_utf16().collect();
        let mut result: usize = 0;
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            env_str.as_ptr(),
            SMTO_ABORTIFHUNG,
            1000,
            &mut result,
        );
    }
}
