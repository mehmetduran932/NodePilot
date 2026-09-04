//! High-performance workspace filesystem scanner.
//!
//! Prunes heavy directories (node_modules, target, dist, .git, etc.) at the root
//! of the branch to maintain high traversal speed across tens of thousands of files.

use std::path::Path;
use walkdir::WalkDir;
use chrono::Utc;
use serde_json::Value;
use nodepilot_core::{
    resolve_project_node, ConfigSourceType, NodePilotError, ProjectRecord
};
use nodepilot_compatibility::evaluate_compatibility;
use crate::framework::detect_framework;
use crate::package_manager::detect_package_manager;
use crate::monorepo::detect_monorepo_root;

const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    "target",
    "coverage",
    ".next",
    ".angular",
    ".cache",
    "out",
    "vendor",
    ".turbo",
    ".idea",
    ".vscode",
    "bin",
    "obj",
];

/// Scans a workspace directory recursively and returns all discovered Node.js projects.
pub fn scan_workspace(
    workspace_root: &Path,
    global_default: Option<&str>,
    max_depth: usize,
) -> Result<Vec<ProjectRecord>, NodePilotError> {
    if !workspace_root.is_dir() {
        return Err(NodePilotError::ProjectNotFound(workspace_root.to_path_buf()));
    }

    let mut projects = Vec::new();
    let walker = WalkDir::new(workspace_root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter();

    for entry in walker.filter_entry(|e| should_traverse(e)) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_dir() {
            continue;
        }

        let dir_path = entry.path();
        let pkg_path = dir_path.join("package.json");

        // We identify a project if package.json is present
        if pkg_path.is_file() {
            if let Some(record) = analyze_project(dir_path, workspace_root, global_default) {
                projects.push(record);
            }
        }
    }

    // Sort projects alphabetically by name
    projects.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(projects)
}

fn should_traverse(entry: &walkdir::DirEntry) -> bool {
    if entry.file_type().is_dir() {
        let name = entry.file_name().to_string_lossy();
        for excluded in EXCLUDED_DIRS {
            if name.eq_ignore_ascii_case(excluded) {
                return false;
            }
        }
    }
    true
}

fn analyze_project(
    project_dir: &Path,
    workspace_root: &Path,
    global_default: Option<&str>,
) -> Option<ProjectRecord> {
    let pkg_path = project_dir.join("package.json");
    let pkg_json: Option<Value> = std::fs::read_to_string(&pkg_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok());

    // Project Name: from package.json or folder name
    let project_name = pkg_json
        .as_ref()
        .and_then(|p| p.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| {
            project_dir
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".into())
        });

    // Detect framework
    let fw_info = detect_framework(project_dir, pkg_json.as_ref());

    // Detect package manager
    let package_manager = detect_package_manager(project_dir, pkg_json.as_ref());

    // Detect engines.node
    let node_requirements = pkg_json.as_ref().and_then(|p| {
        p.get("engines")
            .and_then(|e| e.get("node"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    });

    // Resolve active Node version and configuration source
    let resolution = resolve_project_node(project_dir, global_default);
    let (assigned_node, config_source, config_source_path, is_inherited) = match resolution {
        Some(res) => (
            Some(res.raw_version),
            res.source_type,
            Some(res.source_path),
            res.is_inherited,
        ),
        None => (None, ConfigSourceType::None, None, false),
    };

    // Evaluate compatibility
    let compat = evaluate_compatibility(
        &fw_info.framework,
        fw_info.framework_version.as_deref(),
        assigned_node.as_deref(),
        node_requirements.as_deref(),
    );

    // Git repository check
    let has_git = project_dir.join(".git").exists() || has_git_in_ancestors(project_dir, workspace_root);

    // Monorepo check
    let monorepo = detect_monorepo_root(project_dir, pkg_json.as_ref());
    let is_monorepo_root = monorepo.is_some();

    let id = format!("{:x}", md5_hash(&project_dir.to_string_lossy()));

    Some(ProjectRecord {
        id,
        name: project_name,
        path: project_dir.to_path_buf(),
        workspace_root: workspace_root.to_path_buf(),
        framework: fw_info.framework,
        framework_version: fw_info.framework_version,
        package_manager,
        node_requirements,
        assigned_node,
        config_source,
        config_source_path,
        is_inherited,
        compatibility_status: compat.status,
        compatibility_message: Some(compat.message),
        recommended_node: compat.recommended_node,
        has_git,
        is_monorepo_root,
        monorepo_root_path: None,
        last_scanned: Utc::now(),
    })
}

fn has_git_in_ancestors(dir: &Path, stop_at: &Path) -> bool {
    let mut current = dir.parent();
    while let Some(c) = current {
        if c.join(".git").exists() {
            return true;
        }
        if c == stop_at {
            break;
        }
        current = c.parent();
    }
    false
}

fn md5_hash(input: &str) -> u128 {
    // Simple fast stable hash for project ID
    let mut hash: u128 = 0xcbf29ce484222325;
    for byte in input.as_bytes() {
        hash ^= *byte as u128;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
