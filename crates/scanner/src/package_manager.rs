//! Package manager detection from lockfiles and package.json fields.

use std::path::Path;
use serde_json::Value;
use nodepilot_core::PackageManager;

pub fn detect_package_manager(project_dir: &Path, pkg_json: Option<&Value>) -> PackageManager {
    // 1. Check explicit "packageManager" property in package.json
    if let Some(pkg) = pkg_json {
        if let Some(pm_str) = pkg.get("packageManager").and_then(|v| v.as_str()) {
            let lower = pm_str.to_lowercase();
            if lower.starts_with("pnpm") {
                return PackageManager::Pnpm;
            } else if lower.starts_with("yarn") {
                return PackageManager::Yarn;
            } else if lower.starts_with("bun") {
                return PackageManager::Bun;
            } else if lower.starts_with("npm") {
                return PackageManager::Npm;
            }
        }
    }

    // 2. Check lockfiles in project directory
    if project_dir.join("bun.lock").is_file() || project_dir.join("bun.lockb").is_file() {
        return PackageManager::Bun;
    }
    if project_dir.join("pnpm-lock.yaml").is_file() {
        return PackageManager::Pnpm;
    }
    if project_dir.join("yarn.lock").is_file() {
        return PackageManager::Yarn;
    }
    if project_dir.join("package-lock.json").is_file() {
        return PackageManager::Npm;
    }

    // Default to Npm for Node projects
    PackageManager::Npm
}
