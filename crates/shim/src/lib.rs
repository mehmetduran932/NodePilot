//! NodePilot routing shim management and installation.

use std::path::Path;
use nodepilot_core::{NodePilotError, NodePilotPaths};

/// List of standard and common developer tools to shim by default
pub const DEFAULT_SHIMMED_TOOLS: &[&str] = &[
    "node", "npm", "npx", "corepack",
    "ng", "yarn", "pnpm", "tsc", "vite", "next", "nest", "turbo", "vue",
];

/// Deploys shims for standard tools and common CLI tools into `paths.bin_dir`.
pub fn install_shims(paths: &NodePilotPaths, shim_binary_path: &Path) -> Result<(), NodePilotError> {
    std::fs::create_dir_all(&paths.bin_dir)?;

    #[cfg(target_os = "windows")]
    {
        for tool in DEFAULT_SHIMMED_TOOLS {
            install_single_shim_windows(paths, shim_binary_path, tool)?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::symlink;
        for tool in DEFAULT_SHIMMED_TOOLS {
            let dest = paths.bin_dir.join(tool);
            let _ = std::fs::remove_file(&dest);
            if let Err(_) = symlink(shim_binary_path, &dest) {
                let _ = std::fs::copy(shim_binary_path, &dest);
            }
        }
    }

    Ok(())
}

/// Installs or updates a single tool shim on Windows
#[cfg(target_os = "windows")]
pub fn install_single_shim_windows(
    paths: &NodePilotPaths,
    shim_binary_path: &Path,
    tool: &str,
) -> Result<(), NodePilotError> {
    let dest_exe = paths.bin_dir.join(format!("{}.exe", tool));
    let _ = std::fs::copy(shim_binary_path, &dest_exe);

    // Provide .cmd wrapper for cmd.exe and batch compatibility
    let cmd_file = paths.bin_dir.join(format!("{}.cmd", tool));
    let cmd_content = format!(
        "@ECHO off\r\n\"%~dp0{}.exe\" %*\r\n",
        tool
    );
    let _ = std::fs::write(&cmd_file, cmd_content);
    Ok(())
}
