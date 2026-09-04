//! Ultra-fast command router for node, npm, npx, and corepack.

use std::path::PathBuf;
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

    // Determine target command name (node, npm, npx, corepack)
    let tool_name = if exe_stem == "nodepilot-shim" {
        // If invoked directly as nodepilot-shim, check first arg or default to node
        if args.len() > 1 && ["node", "npm", "npx", "corepack"].contains(&args[1].as_str()) {
            &args[1]
        } else {
            "node"
        }
    } else if ["node", "npm", "npx", "corepack"].contains(&exe_stem.as_str()) {
        &exe_stem
    } else {
        "node"
    };

    // Forwarding arguments (slice past tool name if invoked as nodepilot-shim <tool>)
    let forward_args: Vec<String> = if exe_stem == "nodepilot-shim" && args.len() > 1 && args[1] == tool_name {
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
    let target_bin = get_tool_path(&paths, clean_version, tool_name);

    if !target_bin.is_file() {
        eprintln!("NodePilot: Project requires Node {}, but it is not installed.", version);
        eprintln!("Run:");
        eprintln!("  nodepilot install {}", version);
        std::process::exit(1);
    }

    // Prepare updated PATH with active runtime bin prepended
    let runtime_bin_dir = paths.version_bin_dir(clean_version);
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path_sep = if cfg!(windows) { ";" } else { ":" };
    let new_path = format!("{}{}{}", runtime_bin_dir.to_string_lossy(), path_sep, original_path);

    #[cfg(target_os = "windows")]
    {
        let status = Command::new(&target_bin)
            .args(&forward_args)
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

fn get_tool_path(paths: &NodePilotPaths, version: &str, tool: &str) -> PathBuf {
    match tool {
        "npm" => paths.npm_binary_path(version),
        "npx" => paths.npx_binary_path(version),
        "corepack" => paths.corepack_binary_path(version),
        _ => paths.node_binary_path(version),
    }
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
