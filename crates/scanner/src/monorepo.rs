//! Monorepo detection and workspace structure analysis.

use std::path::Path;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct MonorepoInfo {
    pub is_root: bool,
    pub kind: String,
    pub packages_patterns: Vec<String>,
}

/// Detects whether a directory is a monorepo root.
pub fn detect_monorepo_root(dir: &Path, pkg_json: Option<&Value>) -> Option<MonorepoInfo> {
    // 1. pnpm workspaces
    if dir.join("pnpm-workspace.yaml").is_file() {
        return Some(MonorepoInfo {
            is_root: true,
            kind: "pnpm-workspace".into(),
            packages_patterns: vec!["packages/*".into(), "apps/*".into()],
        });
    }

    // 2. Nx
    if dir.join("nx.json").is_file() {
        return Some(MonorepoInfo {
            is_root: true,
            kind: "Nx".into(),
            packages_patterns: vec!["apps/*".into(), "libs/*".into()],
        });
    }

    // 3. Lerna
    if dir.join("lerna.json").is_file() {
        return Some(MonorepoInfo {
            is_root: true,
            kind: "Lerna".into(),
            packages_patterns: vec!["packages/*".into()],
        });
    }

    // 4. npm / Yarn workspaces in package.json
    if let Some(pkg) = pkg_json {
        if let Some(workspaces) = pkg.get("workspaces") {
            let mut patterns = Vec::new();
            if let Some(arr) = workspaces.as_array() {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        patterns.push(s.to_string());
                    }
                }
            } else if let Some(packages) = workspaces.get("packages").and_then(|p| p.as_array()) {
                for item in packages {
                    if let Some(s) = item.as_str() {
                        patterns.push(s.to_string());
                    }
                }
            }

            if !patterns.is_empty() {
                return Some(MonorepoInfo {
                    is_root: true,
                    kind: "npm/yarn-workspaces".into(),
                    packages_patterns: patterns,
                });
            }
        }
    }

    None
}
