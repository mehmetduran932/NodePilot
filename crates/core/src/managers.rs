//! Non-destructive read-only detection of existing Node managers and runtime environments.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedManager {
    pub name: String,
    pub description: String,
    pub path: Option<PathBuf>,
    pub active_in_path: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentReport {
    pub existing_managers: Vec<DetectedManager>,
    pub system_node_path: Option<PathBuf>,
    pub system_node_version: Option<String>,
    pub has_potential_conflicts: bool,
    pub path_entries: Vec<String>,
    pub recommended_mode: String,
}

/// Inspects the environment non-destructively for existing Node.js managers.
pub fn inspect_environment() -> EnvironmentReport {
    let mut managers = Vec::new();
    let path_var = std::env::var("PATH").unwrap_or_default();
    let path_separator = if cfg!(windows) { ';' } else { ':' };
    let path_entries: Vec<String> = path_var
        .split(path_separator)
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    // 1. fnm
    let fnm_in_path = check_binary_in_path("fnm", &path_entries);
    let fnm_dir = std::env::var("FNM_DIR").ok().map(PathBuf::from);
    if fnm_in_path.is_some() || fnm_dir.is_some() {
        managers.push(DetectedManager {
            name: "fnm".into(),
            description: "Fast Node Manager (fnm)".into(),
            path: fnm_in_path.or(fnm_dir),
            active_in_path: true,
            version: None,
        });
    }

    // 2. nvm / nvm-windows
    let nvm_home = std::env::var("NVM_HOME").ok().map(PathBuf::from);
    let nvm_dir = std::env::var("NVM_DIR").ok().map(PathBuf::from);
    let nvm_in_path = check_binary_in_path("nvm", &path_entries);
    if nvm_home.is_some() || nvm_dir.is_some() || nvm_in_path.is_some() {
        let is_win = cfg!(windows) || nvm_home.is_some();
        managers.push(DetectedManager {
            name: if is_win { "nvm-windows".into() } else { "nvm".into() },
            description: "Node Version Manager".into(),
            path: nvm_home.or(nvm_dir).or(nvm_in_path),
            active_in_path: true,
            version: None,
        });
    }

    // 3. Volta
    let volta_home = std::env::var("VOLTA_HOME").ok().map(PathBuf::from);
    let volta_in_path = check_binary_in_path("volta", &path_entries);
    if volta_home.is_some() || volta_in_path.is_some() {
        managers.push(DetectedManager {
            name: "Volta".into(),
            description: "The Hassle-Free JavaScript Tool Manager".into(),
            path: volta_home.or(volta_in_path),
            active_in_path: true,
            version: None,
        });
    }

    // 4. mise
    let mise_in_path = check_binary_in_path("mise", &path_entries);
    if mise_in_path.is_some() {
        managers.push(DetectedManager {
            name: "mise".into(),
            description: "mise-en-place polyglot tool manager".into(),
            path: mise_in_path,
            active_in_path: true,
            version: None,
        });
    }

    // 5. asdf
    let asdf_dir = std::env::var("ASDF_DIR").ok().map(PathBuf::from);
    let asdf_in_path = check_binary_in_path("asdf", &path_entries);
    if asdf_dir.is_some() || asdf_in_path.is_some() {
        managers.push(DetectedManager {
            name: "asdf".into(),
            description: "Extendable version manager with plugins".into(),
            path: asdf_dir.or(asdf_in_path),
            active_in_path: true,
            version: None,
        });
    }

    // 6. Homebrew Node (macOS / Linux)
    #[cfg(not(target_os = "windows"))]
    {
        let brew_node = PathBuf::from("/opt/homebrew/bin/node");
        let brew_node_intel = PathBuf::from("/usr/local/bin/node");
        if brew_node.exists() || brew_node_intel.exists() {
            managers.push(DetectedManager {
                name: "Homebrew Node".into(),
                description: "Node installed via Homebrew".into(),
                path: if brew_node.exists() { Some(brew_node) } else { Some(brew_node_intel) },
                active_in_path: true,
                version: None,
            });
        }
    }

    // 7. System Node check
    let node_bin = check_binary_in_path("node", &path_entries);
    let mut system_node_version = None;
    if let Some(ref p) = node_bin {
        // Quick read-only query node -v
        if let Ok(output) = std::process::Command::new(p).arg("-v").output() {
            if output.status.success() {
                system_node_version = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
    }

    let has_potential_conflicts = !managers.is_empty();
    let recommended_mode = if has_potential_conflicts {
        "Safe Coexistence Mode".into()
    } else {
        "Direct Shell Integration".into()
    };

    EnvironmentReport {
        existing_managers: managers,
        system_node_path: node_bin,
        system_node_version,
        has_potential_conflicts,
        path_entries,
        recommended_mode,
    }
}

fn check_binary_in_path(bin_name: &str, path_entries: &[String]) -> Option<PathBuf> {
    let target_names: Vec<String> = if cfg!(windows) {
        vec![
            format!("{}.exe", bin_name),
            format!("{}.cmd", bin_name),
            format!("{}.bat", bin_name),
            bin_name.to_string(),
        ]
    } else {
        vec![bin_name.to_string()]
    };

    for entry in path_entries {
        let dir = PathBuf::from(entry);
        for target in &target_names {
            let candidate = dir.join(target);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
