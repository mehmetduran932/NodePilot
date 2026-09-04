//! State storage for workspaces and discovered projects cache.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use crate::error::NodePilotError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CompatibilityStatus {
    Compatible,
    Incompatible,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfigSourceType {
    LocalOverride,       // .nodepilot.local
    Nvmrc,               // .nvmrc
    NodeVersion,         // .node-version
    Volta,               // package.json volta.node
    Engines,             // package.json engines.node
    ToolVersions,        // .tool-versions
    Mise,                // mise.toml
    InheritedLocal,      // Parent .nodepilot.local
    InheritedOther,      // Parent other config
    GlobalDefault,       // Global settings default
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub workspace_root: PathBuf,
    pub framework: String,
    pub framework_version: Option<String>,
    pub package_manager: PackageManager,
    pub node_requirements: Option<String>,
    pub assigned_node: Option<String>,
    pub config_source: ConfigSourceType,
    pub config_source_path: Option<PathBuf>,
    pub is_inherited: bool,
    pub compatibility_status: CompatibilityStatus,
    pub compatibility_message: Option<String>,
    pub recommended_node: Option<String>,
    pub has_git: bool,
    pub is_monorepo_root: bool,
    pub monorepo_root_path: Option<PathBuf>,
    pub last_scanned: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub workspaces: Vec<PathBuf>,
    pub projects: Vec<ProjectRecord>,
    pub last_scan_time: Option<DateTime<Utc>>,
}

impl AppState {
    /// Loads application state from disk.
    pub fn load(path: &Path) -> Result<Self, NodePilotError> {
        if !path.exists() {
            let defaults = Self::default();
            defaults.save(path)?;
            return Ok(defaults);
        }

        let content = std::fs::read_to_string(path)?;
        let state = serde_json::from_str::<Self>(&content).unwrap_or_default();
        Ok(state)
    }

    /// Saves application state to disk atomically.
    pub fn save(&self, path: &Path) -> Result<(), NodePilotError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, json)?;
        std::fs::rename(temp_path, path)?;
        Ok(())
    }

    /// Add a workspace path if not already present.
    pub fn add_workspace(&mut self, path: PathBuf) -> bool {
        let normalized = crate::paths::normalize_path(&path);
        if !self.workspaces.iter().any(|w| w == &normalized) {
            self.workspaces.push(normalized);
            true
        } else {
            false
        }
    }

    /// Remove a workspace path.
    pub fn remove_workspace(&mut self, path: &Path) -> bool {
        let normalized = crate::paths::normalize_path(path);
        let before_len = self.workspaces.len();
        self.workspaces.retain(|w| w != &normalized && w != path);
        // Also remove projects belonging to this workspace
        self.projects.retain(|p| p.workspace_root != normalized && p.workspace_root != path);
        self.workspaces.len() < before_len
    }
}
