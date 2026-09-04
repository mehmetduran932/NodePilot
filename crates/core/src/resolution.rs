//! Project Node version resolution engine.
//!
//! Hierarchy walk with modular configuration file detection:
//! 1. .nodepilot.local
//! 2. .nvmrc
//! 3. .node-version
//! 4. package.json volta.node
//! 5. package.json engines.node
//! 6. .tool-versions
//! 7. mise.toml
//! Recursively walks parent directories up to root before falling back to global default.

use std::path::{Path, PathBuf};
use crate::constants::LOCAL_METADATA_FILENAME;
use crate::state::ConfigSourceType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionResult {
    pub raw_version: String,
    pub source_type: ConfigSourceType,
    pub source_path: PathBuf,
    pub is_inherited: bool,
}

/// Resolves Node version starting at `start_dir` and ascending parents.
pub fn resolve_project_node(start_dir: &Path, global_default: Option<&str>) -> Option<ResolutionResult> {
    let mut current_opt = Some(start_dir);
    let mut depth = 0;

    while let Some(current) = current_opt {
        if let Some(res) = resolve_directory_node(current) {
            let is_inherited = depth > 0;
            let final_source_type = if is_inherited {
                match res.source_type {
                    ConfigSourceType::LocalOverride => ConfigSourceType::InheritedLocal,
                    _ => ConfigSourceType::InheritedOther,
                }
            } else {
                res.source_type
            };

            return Some(ResolutionResult {
                raw_version: res.raw_version,
                source_type: final_source_type,
                source_path: res.source_path,
                is_inherited,
            });
        }

        depth += 1;
        current_opt = current.parent();
    }

    // Fall back to global default if configured
    if let Some(default_ver) = global_default {
        return Some(ResolutionResult {
            raw_version: default_ver.to_string(),
            source_type: ConfigSourceType::GlobalDefault,
            source_path: PathBuf::from("global_settings"),
            is_inherited: false,
        });
    }

    None
}

/// Checks a single directory for Node version configuration files in order of priority.
pub fn resolve_directory_node(dir: &Path) -> Option<ResolutionResult> {
    // 1. .nodepilot.local
    let local_file = dir.join(LOCAL_METADATA_FILENAME);
    if local_file.is_file() {
        if let Ok(content) = std::fs::read_to_string(&local_file) {
            let ver = clean_version_string(&content);
            if !ver.is_empty() {
                return Some(ResolutionResult {
                    raw_version: ver,
                    source_type: ConfigSourceType::LocalOverride,
                    source_path: local_file,
                    is_inherited: false,
                });
            }
        }
    }

    // 2. .nvmrc
    let nvmrc_file = dir.join(".nvmrc");
    if nvmrc_file.is_file() {
        if let Ok(content) = std::fs::read_to_string(&nvmrc_file) {
            let ver = clean_version_string(&content);
            if !ver.is_empty() {
                return Some(ResolutionResult {
                    raw_version: ver,
                    source_type: ConfigSourceType::Nvmrc,
                    source_path: nvmrc_file,
                    is_inherited: false,
                });
            }
        }
    }

    // 3. .node-version
    let node_version_file = dir.join(".node-version");
    if node_version_file.is_file() {
        if let Ok(content) = std::fs::read_to_string(&node_version_file) {
            let ver = clean_version_string(&content);
            if !ver.is_empty() {
                return Some(ResolutionResult {
                    raw_version: ver,
                    source_type: ConfigSourceType::NodeVersion,
                    source_path: node_version_file,
                    is_inherited: false,
                });
            }
        }
    }

    // 4 & 5: package.json (volta.node or engines.node)
    let pkg_file = dir.join("package.json");
    if pkg_file.is_file() {
        if let Ok(content) = std::fs::read_to_string(&pkg_file) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                // 4. volta.node
                if let Some(volta_node) = val.get("volta").and_then(|v| v.get("node")).and_then(|n| n.as_str()) {
                    let ver = clean_version_string(volta_node);
                    if !ver.is_empty() {
                        return Some(ResolutionResult {
                            raw_version: ver,
                            source_type: ConfigSourceType::Volta,
                            source_path: pkg_file,
                            is_inherited: false,
                        });
                    }
                }

                // 5. engines.node
                if let Some(engines_node) = val.get("engines").and_then(|e| e.get("node")).and_then(|n| n.as_str()) {
                    let ver = clean_version_string(engines_node);
                    if !ver.is_empty() {
                        return Some(ResolutionResult {
                            raw_version: ver,
                            source_type: ConfigSourceType::Engines,
                            source_path: pkg_file,
                            is_inherited: false,
                        });
                    }
                }
            }
        }
    }

    // 6. .tool-versions (asdf / mise)
    let tool_versions = dir.join(".tool-versions");
    if tool_versions.is_file() {
        if let Ok(content) = std::fs::read_to_string(&tool_versions) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("nodejs ") || trimmed.starts_with("node ") {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let ver = clean_version_string(parts[1]);
                        if !ver.is_empty() {
                            return Some(ResolutionResult {
                                raw_version: ver,
                                source_type: ConfigSourceType::ToolVersions,
                                source_path: tool_versions,
                                is_inherited: false,
                            });
                        }
                    }
                }
            }
        }
    }

    // 7. mise.toml
    let mise_file = dir.join("mise.toml");
    if mise_file.is_file() {
        if let Ok(content) = std::fs::read_to_string(&mise_file) {
            if let Some(ver) = parse_mise_node_version(&content) {
                return Some(ResolutionResult {
                    raw_version: ver,
                    source_type: ConfigSourceType::Mise,
                    source_path: mise_file,
                    is_inherited: false,
                });
            }
        }
    }

    None
}

/// Trims whitespace, comments, quotes, and standardizes version strings.
pub fn clean_version_string(raw: &str) -> String {
    let first_line = raw.lines().next().unwrap_or("").trim();
    let without_comments = first_line.split('#').next().unwrap_or("").trim();
    let stripped = without_comments.trim_matches(|c| c == '"' || c == '\'' || c == ' ');
    stripped.to_string()
}

/// Parse node version from mise.toml (simple TOML key search without heavy dependencies)
fn parse_mise_node_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("node =") || trimmed.starts_with("nodejs =") {
            if let Some((_, val)) = trimmed.split_once('=') {
                let cleaned = clean_version_string(val);
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
    }
    None
}
