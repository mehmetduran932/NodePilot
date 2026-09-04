//! NodePilot routing shim management and installation.

use std::path::Path;
use nodepilot_core::{NodePilotError, NodePilotPaths};

/// Deploys shims for node, npm, npx, and corepack into `paths.bin_dir`.
pub fn install_shims(paths: &NodePilotPaths, shim_binary_path: &Path) -> Result<(), NodePilotError> {
    std::fs::create_dir_all(&paths.bin_dir)?;

    #[cfg(target_os = "windows")]
    {
        // On Windows:
        // We copy nodepilot-shim.exe to node.exe, npm.exe, npx.exe, corepack.exe
        // and also provide .cmd wrappers for cmd.exe compatibility
        let tools = ["node", "npm", "npx", "corepack"];
        for tool in &tools {
            let dest_exe = paths.bin_dir.join(format!("{}.exe", tool));
            // Copy or replace binary
            let _ = std::fs::copy(shim_binary_path, &dest_exe);

            // Also create .cmd wrapper for shells that prioritize .cmd for npm/npx
            if *tool != "node" {
                let cmd_file = paths.bin_dir.join(format!("{}.cmd", tool));
                let cmd_content = format!(
                    "@ECHO off\r\n\"%~dp0{}.exe\" %*\r\n",
                    tool
                );
                let _ = std::fs::write(&cmd_file, cmd_content);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::symlink;
        let tools = ["node", "npm", "npx", "corepack"];
        for tool in &tools {
            let dest = paths.bin_dir.join(tool);
            let _ = std::fs::remove_file(&dest);
            // Create symlink or hard link
            if let Err(_) = symlink(shim_binary_path, &dest) {
                let _ = std::fs::copy(shim_binary_path, &dest);
            }
        }
    }

    Ok(())
}
