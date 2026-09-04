//! Node.js official release metadata index fetcher, cache, and version resolver.

use serde::{Deserialize, Serialize};
use nodepilot_core::{NodePilotError, NodePilotPaths};

const NODE_DIST_INDEX_URL: &str = "https://nodejs.org/dist/index.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRelease {
    pub version: String,
    pub date: String,
    pub files: Vec<String>,
    #[serde(default)]
    pub npm: Option<String>,
    #[serde(default)]
    pub lts: serde_json::Value, // Can be bool (false) or string ("Iron", "Jod")
    #[serde(default)]
    pub security: bool,
}

impl NodeRelease {
    pub fn is_lts(&self) -> bool {
        match &self.lts {
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::String(_) => true,
            _ => false,
        }
    }

    pub fn lts_codename(&self) -> Option<String> {
        match &self.lts {
            serde_json::Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn clean_version(&self) -> String {
        self.version.trim_start_matches('v').to_string()
    }
}

/// Fetches the list of official Node.js releases, using cache if fresh (< 2 hours).
pub async fn fetch_releases(paths: &NodePilotPaths, force_refresh: bool) -> Result<Vec<NodeRelease>, NodePilotError> {
    let cache_file = paths.cache_dir.join("releases_index.json");

    if !force_refresh && cache_file.is_file() {
        if let Ok(metadata) = std::fs::metadata(&cache_file) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() < 7200 { // 2 hours
                        if let Ok(cached_data) = std::fs::read_to_string(&cache_file) {
                            if let Ok(releases) = serde_json::from_str::<Vec<NodeRelease>>(&cached_data) {
                                return Ok(releases);
                            }
                        }
                    }
                }
            }
        }
    }

    let client = reqwest::Client::builder()
        .user_agent("NodePilot-Agent")
        .build()
        .map_err(|e| NodePilotError::ReleaseFetch(e.to_string()))?;

    let response = client
        .get(NODE_DIST_INDEX_URL)
        .send()
        .await
        .map_err(|e| NodePilotError::ReleaseFetch(format!("Failed to connect to nodejs.org: {}", e)))?;

    if !response.status().is_success() {
        return Err(NodePilotError::ReleaseFetch(format!(
            "HTTP {} from nodejs.org/dist/index.json",
            response.status()
        )));
    }

    let releases = response
        .json::<Vec<NodeRelease>>()
        .await
        .map_err(|e| NodePilotError::ReleaseFetch(format!("Failed to parse release index JSON: {}", e)))?;

    // Cache locally
    if let Ok(serialized) = serde_json::to_string(&releases) {
        let _ = std::fs::create_dir_all(&paths.cache_dir);
        let _ = std::fs::write(&cache_file, serialized);
    }

    Ok(releases)
}

/// Resolves a requested version string, alias ("lts", "latest"), or major number ("22") to an exact version string (e.g. "22.18.0").
pub fn resolve_version_alias(query: &str, releases: &[NodeRelease]) -> Result<String, NodePilotError> {
    let q = query.trim().to_lowercase();
    let q_trimmed = q.trim_start_matches('v');

    if q == "latest" {
        return releases
            .first()
            .map(|r| r.clean_version())
            .ok_or_else(|| NodePilotError::VersionParse("No releases found in release index".into()));
    }

    if q == "lts" {
        return releases
            .iter()
            .find(|r| r.is_lts())
            .map(|r| r.clean_version())
            .ok_or_else(|| NodePilotError::VersionParse("No LTS releases found".into()));
    }

    // Check if exact match
    for r in releases {
        let clean = r.clean_version();
        if clean == q_trimmed {
            return Ok(clean);
        }
    }

    // Check if major version match (e.g. "22" matches "22.18.0")
    if let Ok(major) = q_trimmed.parse::<u64>() {
        for r in releases {
            let clean = r.clean_version();
            if let Some(first_part) = clean.split('.').next() {
                if first_part.parse::<u64>() == Ok(major) {
                    return Ok(clean);
                }
            }
        }
    }

    // Check if partial prefix match (e.g. "20.19")
    for r in releases {
        let clean = r.clean_version();
        if clean.starts_with(q_trimmed) {
            return Ok(clean);
        }
    }

    // If not found in index, but user passed valid semver, allow as fallback
    if semver::Version::parse(q_trimmed).is_ok() {
        return Ok(q_trimmed.to_string());
    }

    Err(NodePilotError::VersionParse(format!(
        "Could not resolve Node.js version query '{}' to a known release",
        query
    )))
}

/// Detects the expected archive filename for current platform and architecture.
pub fn get_archive_filename(clean_version: &str) -> Result<String, NodePilotError> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86_64") => Ok(format!("node-v{}-win-x64.zip", clean_version)),
        ("windows", "aarch64") => Ok(format!("node-v{}-win-arm64.zip", clean_version)),
        ("macos", "x86_64") => Ok(format!("node-v{}-darwin-x64.tar.gz", clean_version)),
        ("macos", "aarch64") => Ok(format!("node-v{}-darwin-arm64.tar.gz", clean_version)),
        ("linux", "x86_64") => Ok(format!("node-v{}-linux-x64.tar.gz", clean_version)),
        ("linux", "aarch64") => Ok(format!("node-v{}-linux-arm64.tar.gz", clean_version)),
        _ => Err(NodePilotError::InvalidOperation(format!(
            "Unsupported OS '{}' and architecture '{}'",
            os, arch
        ))),
    }
}
