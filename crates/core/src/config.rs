//! Configuration settings management for NodePilot.

use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::NodePilotError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub scan_on_startup: bool,
    pub auto_check_node_updates: bool,
    pub auto_check_app_updates: bool,
    pub show_prereleases: bool,
    pub preferred_terminal: String,
    pub preferred_editor: String,
    pub safe_mode: bool,
    pub shell_integration_enabled: bool,
    pub telemetry: bool,
    pub theme: String,
    pub global_default_version: Option<String>,
    pub gui_installed: bool,
    pub ask_gui_launch: bool,
    pub auto_install_missing_node: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scan_on_startup: true,
            auto_check_node_updates: true,
            auto_check_app_updates: true,
            show_prereleases: false,
            preferred_terminal: "system".into(),
            preferred_editor: "code".into(),
            safe_mode: true,
            shell_integration_enabled: false,
            telemetry: false, // Strict privacy default
            theme: "system".into(),
            global_default_version: None,
            gui_installed: false,
            ask_gui_launch: true,
            auto_install_missing_node: true,
        }
    }
}

impl Settings {
    /// Loads settings from disk, creating default if not found.
    pub fn load(path: &Path) -> Result<Self, NodePilotError> {
        if !path.exists() {
            let defaults = Self::default();
            defaults.save(path)?;
            return Ok(defaults);
        }

        let content = std::fs::read_to_string(path)?;
        let settings = serde_json::from_str::<Self>(&content).unwrap_or_default();
        Ok(settings)
    }

    /// Saves settings to disk atomically.
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
}
