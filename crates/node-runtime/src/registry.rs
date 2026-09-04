//! Registry for querying and managing locally installed Node.js runtime versions.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use nodepilot_core::{NodePilotError, NodePilotPaths, ProjectRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledVersion {
    pub version: String,
    pub path: PathBuf,
    pub binary_path: PathBuf,
    pub npm_version: Option<String>,
    pub size_bytes: u64,
    pub projects_using_count: usize,
    pub projects_using: Vec<String>,
}

/// Checks if a given version is installed and has a valid executable binary.
pub fn is_version_installed(paths: &NodePilotPaths, version: &str) -> bool {
    let clean = version.trim_start_matches('v');
    let bin_path = paths.node_binary_path(clean);
    bin_path.is_file()
}

/// Lists all currently installed Node.js runtimes in NodePilot.
pub fn list_installed_versions(
    paths: &NodePilotPaths,
    projects: &[ProjectRecord],
) -> Result<Vec<InstalledVersion>, NodePilotError> {
    let mut installed = Vec::new();

    if !paths.versions_dir.is_dir() {
        return Ok(installed);
    }

    let entries = std::fs::read_dir(&paths.versions_dir)?;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if !file_type.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();
        if dir_name.starts_with('.') {
            continue; // Skip hidden or temp directories
        }

        let bin_path = paths.node_binary_path(&dir_name);
        if !bin_path.is_file() {
            continue;
        }

        // Calculate size on disk
        let size_bytes = calculate_dir_size(&entry.path());

        // Count projects actively assigned to this version
        let mut projects_using = Vec::new();
        for proj in projects {
            if let Some(ref assigned) = proj.assigned_node {
                let clean_assigned = assigned.trim_start_matches('v');
                if clean_assigned == dir_name || clean_assigned.starts_with(&format!("{}.", dir_name)) {
                    projects_using.push(proj.name.clone());
                }
            }
        }

        let projects_using_count = projects_using.len();

        installed.push(InstalledVersion {
            version: dir_name,
            path: entry.path(),
            binary_path: bin_path,
            npm_version: None,
            size_bytes,
            projects_using_count,
            projects_using,
        });
    }

    // Sort descending by semver where possible
    installed.sort_by(|a, b| {
        let ver_a = semver::Version::parse(&a.version).ok();
        let ver_b = semver::Version::parse(&b.version).ok();
        match (ver_a, ver_b) {
            (Some(va), Some(vb)) => vb.cmp(&va),
            _ => b.version.cmp(&a.version),
        }
    });

    Ok(installed)
}

fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += calculate_dir_size(&entry.path());
                }
            }
        }
    }
    total
}
