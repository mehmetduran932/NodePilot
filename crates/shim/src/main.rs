//! Ultra-fast command router for node, npm, npx, corepack, and all ecosystem tools (ng, yarn, pnpm, tsc, vite, etc.).

use std::path::{Path, PathBuf};
use std::process::Command;
use nodepilot_core::{
    resolve_project_node, NodePilotPaths, Settings, SHIM_RECURSION_ENV_VAR,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("node"));
    let exe_stem = current_exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("node")
        .to_lowercase();

    // Determine target command name
    let tool_name = if exe_stem == "nodepilot-shim" {
        // If invoked directly as nodepilot-shim, check first arg or default to node
        if args.len() > 1 && !args[1].starts_with('-') {
            args[1].clone()
        } else {
            "node".to_string()
        }
    } else {
        exe_stem
    };

    // Forwarding arguments (slice past tool name if invoked as nodepilot-shim <tool>)
    let forward_args: Vec<String> = if current_exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .eq_ignore_ascii_case("nodepilot-shim")
        && args.len() > 1
        && args[1] == tool_name
    {
        args[2..].to_vec()
    } else {
        args[1..].to_vec()
    };

    // Recursion loop defense
    if std::env::var(SHIM_RECURSION_ENV_VAR).is_ok() {
        eprintln!("NodePilot: Recursive shim invocation detected. Please ensure Node runtime is installed properly.");
        std::process::exit(1);
    }

    let paths = match NodePilotPaths::default() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("NodePilot: Failed to resolve paths: {}", e);
            std::process::exit(1);
        }
    };

    let settings = Settings::load(&paths.settings_file()).unwrap_or_default();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Resolve required Node version
    let resolved = resolve_project_node(&cwd, settings.global_default_version.as_deref());

    let (version, _is_fallback) = match resolved {
        Some(res) => (res.raw_version, false),
        None => {
            // Find any latest installed version as fallback
            if let Some(latest_installed) = find_any_installed_version(&paths) {
                (latest_installed, true)
            } else {
                eprintln!("NodePilot: No Node.js version assigned to this directory, and no versions are installed.");
                eprintln!("Run 'nodepilot install 22' or 'nodepilot assign 22' to configure.");
                std::process::exit(1);
            }
        }
    };

    let clean_version = version.trim_start_matches('v');

    // Ensure Node.js runtime is installed
    let node_bin = paths.node_binary_path(clean_version);
    if !node_bin.is_file() {
        if settings.auto_install_missing_node {
            eprintln!("NodePilot: Project requires Node {}, which is not yet installed.", version);
            eprintln!("NodePilot: Automatically installing Node {} environment...", version);

            let nodepilot_cli = paths.bin_dir.join(if cfg!(windows) { "nodepilot.exe" } else { "nodepilot" });
            let install_cmd = if nodepilot_cli.is_file() {
                nodepilot_cli
            } else {
                PathBuf::from("nodepilot")
            };

            let install_status = Command::new(&install_cmd)
                .arg("install")
                .arg(&version)
                .status();

            if !(install_status.is_ok() && node_bin.is_file()) {
                eprintln!("NodePilot: Automatic installation failed. Please run: nodepilot install {}", version);
                std::process::exit(1);
            }
        } else {
            eprintln!("NodePilot: Project requires Node {}, but it is not installed.", version);
            eprintln!("Run: nodepilot install {}", version);
            std::process::exit(1);
        }
    }

    // Resolve target executable
    let is_standard_tool = ["node", "npm", "npx", "corepack"].contains(&tool_name.as_str());
    let target_bin = if is_standard_tool {
        get_standard_tool_path(&paths, clean_version, &tool_name)
    } else {
        match resolve_external_tool(&paths, clean_version, &tool_name, &cwd, &settings) {
            Some(p) => p,
            None => {
                eprintln!("NodePilot: Command '{}' is not recognized or not installed in Node {} environment.", tool_name, clean_version);
                eprintln!("Tip: Install it locally in your project or run 'npm install -g {}' to make it available.", tool_name);
                std::process::exit(1);
            }
        }
    };

    // Prepare updated PATH with project local node_modules/.bin AND active runtime bin prepended
    let runtime_bin_dir = paths.version_bin_dir(clean_version);
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path_sep = if cfg!(windows) { ";" } else { ":" };

    let mut path_additions = Vec::new();
    if let Some(local_nm_bin) = find_local_node_modules_bin_dir(&cwd) {
        path_additions.push(local_nm_bin.to_string_lossy().to_string());
    }
    path_additions.push(runtime_bin_dir.to_string_lossy().to_string());

    let additions_str = path_additions.join(path_sep);
    let new_path = if original_path.starts_with(&additions_str) {
        original_path
    } else {
        format!("{}{}{}", additions_str, path_sep, original_path)
    };

    #[cfg(target_os = "windows")]
    {
        let is_cmd_or_bat = target_bin
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat"))
            .unwrap_or(false);

        let mut cmd = if is_cmd_or_bat {
            let mut c = Command::new("cmd.exe");
            c.arg("/c").arg(&target_bin);
            c.args(&forward_args);
            c
        } else {
            let mut c = Command::new(&target_bin);
            c.args(&forward_args);
            c
        };

        let status = cmd
            .env(SHIM_RECURSION_ENV_VAR, "1")
            .env("PATH", new_path)
            .status();

        match status {
            Ok(s) => std::process::exit(s.code().unwrap_or(0)),
            Err(e) => {
                eprintln!("NodePilot: Failed to execute {}: {}", target_bin.display(), e);
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new(&target_bin)
            .args(&forward_args)
            .env(SHIM_RECURSION_ENV_VAR, "1")
            .env("PATH", new_path)
            .exec();

        eprintln!("NodePilot: Failed to exec {}: {}", target_bin.display(), err);
        std::process::exit(1);
    }
}

fn get_standard_tool_path(paths: &NodePilotPaths, version: &str, tool: &str) -> PathBuf {
    match tool {
        "npm" => paths.npm_binary_path(version),
        "npx" => paths.npx_binary_path(version),
        "corepack" => paths.corepack_binary_path(version),
        _ => paths.node_binary_path(version),
    }
}

/// Resolves an ecosystem tool (ng, yarn, pnpm, tsc, vite, etc.)
fn resolve_external_tool(
    paths: &NodePilotPaths,
    version: &str,
    tool: &str,
    cwd: &Path,
    settings: &Settings,
) -> Option<PathBuf> {
    // 1. Check local project node_modules/.bin (highest priority for project-specific tooling)
    if let Some(local_bin) = find_local_node_modules_bin(cwd, tool) {
        return Some(local_bin);
    }

    // 2. Check active NodePilot version directory
    let version_dir = paths.version_dir(version);
    let candidate_names = if cfg!(windows) {
        vec![
            format!("{}.cmd", tool),
            format!("{}.exe", tool),
            format!("{}.bat", tool),
            tool.to_string(),
        ]
    } else {
        vec![tool.to_string()]
    };

    for name in &candidate_names {
        let p = version_dir.join(name);
        if p.is_file() {
            return Some(p);
        }
        let p_bin = version_dir.join("bin").join(name);
        if p_bin.is_file() {
            return Some(p_bin);
        }
        let p_nm = version_dir.join("node_modules").join(".bin").join(name);
        if p_nm.is_file() {
            return Some(p_nm);
        }
    }

    // 3. Fallback: Check existing NVM or other NodePilot versions for this tool and auto-sync
    if let Some(migrated_bin) = try_import_tool_from_nvm_or_versions(paths, version, tool) {
        return Some(migrated_bin);
    }

    // 4. Auto-install fallback via active npm
    if settings.auto_install_missing_node {
        if let Some(pkg) = map_tool_to_npm_package(tool) {
            eprintln!("NodePilot: Tool '{}' ({}) not found in Node {}. Auto-installing globally...", tool, pkg, version);
            let npm_cmd = paths.npm_binary_path(version);

            #[cfg(target_os = "windows")]
            let mut install_cmd = {
                let mut c = Command::new("cmd.exe");
                c.arg("/c").arg(&npm_cmd).arg("install").arg("-g").arg(pkg);
                c
            };

            #[cfg(not(target_os = "windows"))]
            let mut install_cmd = {
                let mut c = Command::new(&npm_cmd);
                c.arg("install").arg("-g").arg(pkg);
                c
            };

            let _ = install_cmd.status();

            // Re-check version directory
            for name in &candidate_names {
                let p = version_dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
                let p_bin = version_dir.join("bin").join(name);
                if p_bin.is_file() {
                    return Some(p_bin);
                }
            }
        }
    }

    None
}

fn map_tool_to_npm_package(tool: &str) -> Option<&'static str> {
    match tool {
        "ng" => Some("@angular/cli"),
        "yarn" => Some("yarn"),
        "pnpm" => Some("pnpm"),
        "tsc" => Some("typescript"),
        "vite" => Some("vite"),
        "next" => Some("next"),
        "nest" => Some("@nestjs/cli"),
        "vue" => Some("@vue/cli"),
        "turbo" => Some("turbo"),
        "nodemon" => Some("nodemon"),
        _ => None,
    }
}

/// Locates local node_modules/.bin directory walking up from cwd
fn find_local_node_modules_bin_dir(cwd: &Path) -> Option<PathBuf> {
    let mut current = Some(cwd);
    while let Some(dir) = current {
        let bin_dir = dir.join("node_modules").join(".bin");
        if bin_dir.is_dir() {
            return Some(bin_dir);
        }
        current = dir.parent();
    }
    None
}

/// Locates a tool binary in local node_modules/.bin walking up from cwd
fn find_local_node_modules_bin(cwd: &Path, tool: &str) -> Option<PathBuf> {
    let mut current = Some(cwd);
    while let Some(dir) = current {
        let bin_dir = dir.join("node_modules").join(".bin");
        if bin_dir.is_dir() {
            #[cfg(target_os = "windows")]
            {
                let candidates = [
                    bin_dir.join(format!("{}.cmd", tool)),
                    bin_dir.join(format!("{}.exe", tool)),
                    bin_dir.join(format!("{}.bat", tool)),
                    bin_dir.join(tool),
                ];
                for candidate in candidates {
                    if candidate.is_file() {
                        return Some(candidate);
                    }
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                let candidate = bin_dir.join(tool);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        current = dir.parent();
    }
    None
}

/// Auto-discovers and syncs a tool from existing NVM or other NodePilot versions
fn try_import_tool_from_nvm_or_versions(
    paths: &NodePilotPaths,
    target_version: &str,
    tool: &str,
) -> Option<PathBuf> {
    let dest_dir = paths.version_dir(target_version);

    #[cfg(target_os = "windows")]
    {
        // 1. Check NVM directory at %APPDATA%\nvm
        if let Ok(appdata) = std::env::var("APPDATA") {
            let nvm_dir = PathBuf::from(appdata).join("nvm");
            if nvm_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let tool_cmd = path.join(format!("{}.cmd", tool));
                            if tool_cmd.is_file() {
                                // Found in NVM! Copy into NodePilot version directory
                                let _ = std::fs::copy(&tool_cmd, dest_dir.join(format!("{}.cmd", tool)));
                                let _ = std::fs::copy(path.join(format!("{}.ps1", tool)), dest_dir.join(format!("{}.ps1", tool)));
                                let _ = std::fs::copy(path.join(tool), dest_dir.join(tool));

                                // If tool has node_modules (e.g. @angular for ng)
                                copy_matching_tool_node_modules(&path.join("node_modules"), &dest_dir.join("node_modules"), tool);

                                let final_bin = dest_dir.join(format!("{}.cmd", tool));
                                if final_bin.is_file() {
                                    return Some(final_bin);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Check other NodePilot installed versions
    if let Ok(entries) = std::fs::read_dir(&paths.versions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.file_name().map(|n| n != target_version).unwrap_or(false) {
                let candidate = if cfg!(windows) {
                    path.join(format!("{}.cmd", tool))
                } else {
                    path.join("bin").join(tool)
                };
                if candidate.is_file() {
                    let dest_file = if cfg!(windows) {
                        dest_dir.join(format!("{}.cmd", tool))
                    } else {
                        dest_dir.join("bin").join(tool)
                    };
                    let _ = std::fs::copy(&candidate, &dest_file);
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::fs::copy(path.join(format!("{}.ps1", tool)), dest_dir.join(format!("{}.ps1", tool)));
                        let _ = std::fs::copy(path.join(tool), dest_dir.join(tool));
                        copy_matching_tool_node_modules(&path.join("node_modules"), &dest_dir.join("node_modules"), tool);
                    }
                    if dest_file.is_file() {
                        return Some(dest_file);
                    }
                }
            }
        }
    }

    None
}

/// Helper to copy tool-specific packages from one node_modules directory to another
#[cfg(target_os = "windows")]
fn copy_matching_tool_node_modules(src_nm: &Path, dest_nm: &Path, tool: &str) {
    if !src_nm.is_dir() {
        return;
    }
    let _ = std::fs::create_dir_all(dest_nm);

    // Special package mappings
    let target_dirs: &[&str] = match tool {
        "ng" => &["@angular"],
        "yarn" => &["yarn"],
        "pnpm" => &["pnpm"],
        "tsc" => &["typescript"],
        "nest" => &["@nestjs"],
        "vue" => &["@vue"],
        _ => &[],
    };

    for &dir_name in target_dirs {
        let src_pkg = src_nm.join(dir_name);
        let dest_pkg = dest_nm.join(dir_name);
        if src_pkg.exists() && !dest_pkg.exists() {
            let _ = copy_dir_all(&src_pkg, &dest_pkg);
        }
    }
}

#[cfg(target_os = "windows")]
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn find_any_installed_version(paths: &NodePilotPaths) -> Option<String> {
    if !paths.versions_dir.is_dir() {
        return None;
    }
    let mut versions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&paths.versions_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with('.') && paths.node_binary_path(&name).is_file() {
                    versions.push(name);
                }
            }
        }
    }
    versions.sort_by(|a, b| b.cmp(a));
    versions.into_iter().next()
}
